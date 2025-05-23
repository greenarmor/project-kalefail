<script lang="ts">
    import kale_tractor from '$lib/contracts/kale_tractor';
    import { send } from '$lib/passkeyClient';
    import { scValToNative, xdr } from '@stellar/stellar-sdk';
    import { Api } from '@stellar/stellar-sdk/rpc';
    import type { i128 } from '@stellar/stellar-sdk/contract';
    import PageHeader from '$lib/components/ui/PageHeader.svelte';
    import StatusDefListItem from '$lib/components/ui/StatusDefListItem.svelte';

    import { getToastStore } from '@skeletonlabs/skeleton';
    const toastStore = getToastStore();
    const MAX_PAILS_PER_HARVEST: number = 29;

    let farmerAddress = $state('');
    let harvestablePails: number[] = $state([]);
    let selectedPails: number[] = $state([]);

    let isFetching = $state(false);
    let hasFetched = $state(false);
    let isHarvesting = $state(false);
    let shouldBulkSelect = $state(false);

    let tooManyPailsFound = $derived(harvestablePails.length > MAX_PAILS_PER_HARVEST);
    let bulkSelectChecked = $derived.by(() => {
        if (tooManyPailsFound) {
            return shouldBulkSelect;
        } else {
            return selectedPails.length > 0 && selectedPails.length === harvestablePails.length;
        }
    });
    let somePailsSelected = $derived.by(() => {
        if (tooManyPailsFound) {
            return false;
        } else {
            return selectedPails.length > 0 && selectedPails.length < harvestablePails.length;
        }
    });

    function sliceHarvestablePails(): number[] {
        return harvestablePails.slice(0, MAX_PAILS_PER_HARVEST);
    }

    function selectPailsBatch(event: Event) {
        shouldBulkSelect = (event.target as HTMLInputElement).checked;
        if (shouldBulkSelect) {
            selectedPails = sliceHarvestablePails();
        } else {
            selectedPails = [];
        }
    }

    async function fetchPails() {
        console.log('finding pails');
        isFetching = true;
        try {
            let tractorRes = await fetch(`/api/tractor/${farmerAddress}`);
            let tractorJson = await tractorRes.json();

            if (!tractorRes.ok) {
                throw tractorJson.message;
            }

            harvestablePails = tractorJson.pails;

            if (shouldBulkSelect) {
                selectedPails = sliceHarvestablePails();
            } else {
                selectedPails = [];
            }

            hasFetched = true;
        } catch (err: unknown) {
            console.error('error', err);

            toastStore.trigger({
                message:
                    err instanceof Error
                        ? err.message
                        : 'Something went wrong fetching pails. Please try again later.',
                background: 'variant-filled-error',
            });
        } finally {
            isFetching = false;
        }
    }

    async function harvestPails() {
        console.log('harvesting pails');
        isHarvesting = true;

        try {
            let at = await kale_tractor.harvest({
                farmer: farmerAddress,
                pails: selectedPails,
            });

            if (Api.isSimulationError(at.simulation!)) {
                if (at.simulation.error.includes('Error(Contract, #9)')) {
                    throw 'Pail not found in homestead storage. Ask Elliot about that...';
                } else if (at.simulation.error.includes('Error(Contract, #1)')) {
                    throw 'No pails requested for harvest. Please select some pails first.';
                } else if (at.simulation.error.includes('Error(Contract, #2)')) {
                    throw "No rewards for selected pails. Sorry, I guess you didn't work hard enough?";
                }
                throw '';
            }

            let { returnValue } = await send(at.built!);
            let rewards: i128[] = scValToNative(xdr.ScVal.fromXDR(returnValue, 'base64'));
            let totalReward = rewards.reduce((total, n) => total + n, BigInt(0));

            toastStore.trigger({
                message: `Sweet!! You successfully harvested ${(Number(totalReward) / 10_000_000).toFixed(7)} KALE.`,
                background: 'variant-filled-success',
            });
        } catch (err: unknown) {
            console.error('error', err);

            toastStore.trigger({
                message:
                    err instanceof Error
                        ? err.message
                        : 'Something went wrong harvesting pails. Please try again later.',
                background: 'variant-filled-error',
            });
        } finally {
            isHarvesting = false;
            fetchPails();
        }
    }
</script>

<PageHeader
    title="The Tractor"
    subtitle="Welcome to the heavy duty tractor. Where all your harvest dreams come true!"
/>

<p>
    Sometimes, stuff happens. Your webminer acts up, you have to log out, your dog pukes everywhere.
    Ya know: stuff! It's easy to potentially lose out on your hard-earned <code class="code"
        >KALE</code
    > harvests. Well, fret no more!
</p>

<p>
    The tractor is here to help you reclaim your harvest. You still have to get them before they're
    evicted (they haven't paid the landlord in a while!), but now you can get back to them easier
    than ever.
</p>

<div class="w-full grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="flex flex-col card bg-initial justify-between">
        <header class="card-header">
            <h2 class="h2 text-center">Find Pails</h2>
        </header>
        <section class="p-4 space-y-4">
            <label class="label">
                <span>Find all harvestable pails for...</span>
                <input
                    class="input"
                    type="text"
                    placeholder="Enter address here"
                    bind:value={farmerAddress}
                />
            </label>
        </section>
        <footer class="card-footer text-center">
            <button
                class="btn variant-filled"
                disabled={isFetching || !farmerAddress}
                onclick={fetchPails}>Fetch Pails</button
            >
        </footer>
    </div>
    <div class="card bg-initial">
        <header class="card-header">
            <h2 class="h2 text-center">Harvest Pails</h2>
        </header>
        <section class="p-4 space-y-4">
            <dl class="list-dl">
                <StatusDefListItem
                    title="Tractor Address"
                    value={kale_tractor.options.contractId}
                    isAddress
                />
            </dl>
            {#if harvestablePails.length}
                <p>
                    Please select which pails of <code class="code">KALE</code> you'd like to harvest:
                </p>

                <div class="space-y-2">
                    <label class="flex items-center space-x-2">
                        <input
                            class="checkbox"
                            type="checkbox"
                            onchange={selectPailsBatch}
                            checked={bulkSelectChecked}
                            indeterminate={somePailsSelected}
                        />
                        <p>
                            {#if tooManyPailsFound}
                                <strong>Auto-Select Pails</strong> (max. {MAX_PAILS_PER_HARVEST} per
                                harvest)
                            {:else}
                                <strong>Select All Pails</strong>
                            {/if}
                        </p>
                    </label>

                    <hr class="!border-t-2" />
                    <div class="columns-2">
                        {#each harvestablePails as pail (pail)}
                            <label class="flex items-center space-x-2">
                                <input
                                    class="checkbox"
                                    type="checkbox"
                                    value={pail}
                                    bind:group={selectedPails}
                                    disabled={!selectedPails.includes(pail) &&
                                        selectedPails.length === MAX_PAILS_PER_HARVEST}
                                />
                                <p>{pail}</p>
                            </label>
                        {/each}
                    </div>
                </div>
            {:else if hasFetched}
                <p>No harvestable pails found. Great work!</p>
            {:else}
                <p>Not much to see here... Try fetching pails first.</p>
            {/if}
        </section>
        {#if harvestablePails.length}
            <footer class="card-footer text-center">
                <button
                    class="btn variant-filled"
                    disabled={isHarvesting || isFetching || selectedPails.length === 0}
                    onclick={harvestPails}
                >
                    Harvest KALE
                </button>
            </footer>
        {/if}
    </div>
</div>

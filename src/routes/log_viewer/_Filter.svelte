
<script lang="ts">

    import { fade } from "svelte/transition";
    import { Funnel } from "phosphor-svelte";
    import ChannelList from "../../components/_ChannelList.svelte";
    import { get_all_channel_ids } from "../../lib/network/swtor_channel";
    import SmallButton from "../../lib/buttons/SmallButton.svelte";
    import { click_outside_handler } from "../../lib/click_outside";
    import Tooltip from "../../components/_Tooltip.svelte";
    import type { ChannelDispatcher } from "../../lib/network/settings";

    export let channel_filters: ChannelDispatcher[] = [];
    let show_filters: boolean = false;
    let mouse_over: boolean = false;

    function toggle_show_filters() {

        show_filters = !show_filters;

    }

    function click_outside() {

        if (!mouse_over) {
            show_filters = false;
        }

    }

    function select_all_channels() {

        channel_filters = get_all_channel_ids()
            .map((id) => { return { RegularDispatch: id }});

    }

    function deselect_all_channels() {

        channel_filters = [];

    }

    function on_channel_input(e: CustomEvent<{channels: ChannelDispatcher[]}>) {

        channel_filters = e.detail.channels;

    }

</script>

<Tooltip tooltip_text="Filter out by channels" placement="left">
    <button class="hover:text-gray-400 text-white" on:click={toggle_show_filters} on:mouseenter={() => { mouse_over = true }} on:mouseleave={() => { mouse_over = false }}>
        <Funnel size={26}/>
    </button>
</Tooltip>
{#if show_filters}
    <div class="absolute top-10 z-10 bg-slate-600 p-2 rounded-md shadow-md" transition:fade|local="{{ duration: 250 }}" use:click_outside_handler={click_outside}>
        <ChannelList bind:channels={channel_filters} on:channel_input={on_channel_input}/>
        <div class="flex flex-row gap-2">
            <SmallButton on:click={select_all_channels}>Select All</SmallButton>
            <SmallButton on:click={deselect_all_channels}>Deselect All</SmallButton>
        </div>
    </div>
{/if}
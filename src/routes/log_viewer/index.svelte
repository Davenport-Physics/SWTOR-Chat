
<script lang="ts">

    import { invoke } from "@tauri-apps/api";
    import Flatpickr, { type HookProps } from 'svelte-flatpickr'
    import 'flatpickr/dist/flatpickr.css'
    import { type IChatLogMessage } from "../../lib/network/chat_log_message";
    import { active_character } from "../../lib/network/characters";
    import { SwtorMessage } from "../../lib/network/swtor_message";
    import { afterUpdate } from "svelte";

    let container: HTMLElement | undefined    = undefined;
    let last_message: HTMLElement | undefined = undefined;
    let dates: string[] = [];
    let date_messages: SwtorMessage[] = [];

    function init_dates(callback?: () => void) {

        invoke("get_distinct_dates").then((response) => {

            dates = response as string[];

            if (callback != undefined) {
                callback();
            }

        });

    }

    function on_change(e: CustomEvent<HookProps>) {

        let date = e.detail[1];
        invoke("get_chat_log_from_date", {date}).then((response) => {
            date_messages = (response as IChatLogMessage[]).map((m) => new SwtorMessage(m.message));
        });

    }

    function scroll_to_last_message() {

        if (last_message == undefined) {
            return;
        }

        last_message.scrollIntoView({ behavior: "instant", block: "end" });

    }

    afterUpdate(() => {
        scroll_to_last_message();
    });

    init_dates();


</script>


<div class="px-6">

    <div class="h-8"></div>
    <div class="text-white text-2xl text-center bg-slate-600">Log Viewer</div>
    <div class="h-8"></div>

    <Flatpickr options={{ enable: dates }} on:change={on_change} name="date" placeholder="Select a date" class="outline-none border-2 border-slate-700 rounded-md px-2 text-xl"/>
    <div class="h-6"></div>
    <div bind:this={container} class="flex flex-col h-96 rounded-tr-md border-2 border-slate-700 overflow-y-auto scrollbar scrollbar-thumb-sky-800 scrollbar-track-slate-100 chat-container-background">
        {#each date_messages as message}

            <div bind:this={last_message} class="w-full opacity-100">
                <span class="text-white">[{message.timestamp}]</span>
                <!-- svelte-ignore a11y-click-events-have-key-events -->
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <span class="text-slate-200 cursor-pointer">
                    {message.get_message_from()}
                </span>
                {#each message.get_message_fragments() as fragment}
                    {#if fragment.startsWith("\"") && fragment.endsWith("\"")}
                        <span class="break-words " style="color: white;">{fragment}</span>
                    {:else}
                        <span class="break-words " style="color: {$active_character?.get_channel_color(message.channel.type).to_hex()}">{fragment}</span>
                    {/if}
                {/each}
            </div>
        {/each}
    </div>
</div>
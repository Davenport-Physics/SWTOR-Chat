
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { writable, get } from "svelte/store";

import { toast } from "@zerodevx/svelte-toast";
import { Result } from "./result";
import { init_custom_emotes } from "./network/custom_emote";
import { init_settings } from "./network/settings";
import { init_swtor_message_listener } from "./network/swtor_message";
import { init_active_character } from "./network/characters";
import { settings } from "./network/settings";

export type MessageType = "ButtonEmote" | "ChatMessage";

interface UserCharacterMessages {
    message_type: MessageType;
    character_id?: number;
    messages: string[];
}

export const hooked_in = writable<boolean>(false);

export function init_network() {

    init_settings(() => {
        init_active_character();
    });

    init_hook();
    init_swtor_message_listener();
    init_custom_emotes();

}

function init_hook() {

    invoke("start_swtor_hook");
    listen("swtor_hooked_in", (response: any) => {
        hooked_in.set(response.payload.hooked_in);
    });

}

export async function submit_post(message_type: MessageType, messages: string[]): Promise<Result<[], string>> {

    if (!get(hooked_in)) {
        return Result.error("SWTOR not hooked in. Have you launched the game?");
    }

    messages = messages.map((message) => message.trim());
    
    for (let message of messages) { 
        
        if (message.length == 0) {
            return Result.error("Empty message detected. Please remove it.");
        } else if (message.length > 255) {
            return Result.error("Long message detected. Please shorten it.");
        }

    }

    let character_message: UserCharacterMessages = {
        message_type: message_type,
        character_id: undefined,
        messages: messages
    };

    let t_settings = get(settings);
    let retry: boolean = t_settings.chat.retry_message_submission && t_settings.chat_log.capture_chat_log && message_type != "ButtonEmote";

    try {

        await invoke("submit_actual_post", {retry: retry, characterMessage: character_message});
        return Result.ok([]);

    } catch (error: any) {

        return Result.error(error);
        
    }

}


export function open_link(link: string) {
    invoke("open_link", { link: link });
}
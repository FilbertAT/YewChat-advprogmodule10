use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

#[derive(Clone, PartialEq, Debug)]
pub enum Theme {
    Light,
    Dark,
}

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleEmojiPicker,
    SelectEmoji(String),
    ToggleTheme, // New message for toggling theme
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    show_emoji_picker: bool,
    current_theme: Theme, // New state field for current theme
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            show_emoji_picker: false,
            current_theme: Theme::Light, // Initialize with Light theme
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    if !input.value().is_empty() {
                        let message = WebSocketMessage {
                            message_type: MsgTypes::Message,
                            data: Some(input.value()),
                            data_array: None,
                        };
                        if let Err(e) = self
                            .wss
                            .tx
                            .clone()
                            .try_send(serde_json::to_string(&message).unwrap())
                        {
                            log::debug!("error sending to channel: {:?}", e);
                        }
                        input.set_value("");
                    }
                };
                false
            }
            Msg::ToggleEmojiPicker => {
                self.show_emoji_picker = !self.show_emoji_picker;
                true
            }
            Msg::SelectEmoji(emoji) => {
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let current_value = input.value();
                    input.set_value(&format!("{}{}", current_value, emoji));
                }
                self.show_emoji_picker = false;
                true
            }
            Msg::ToggleTheme => {
                self.current_theme = match self.current_theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                };
                true // Re-render is needed
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_emoji_picker = ctx.link().callback(|_| Msg::ToggleEmojiPicker);
        let toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);
        
        // Common emoji set
        let emojis = vec![
            "😀", "😂", "😍", "🥳", "😎", "🤔", "👍", "❤️", 
            "🔥", "✨", "🎉", "👋", "🙏", "🤗", "😊", "🥰"
        ];

        // Define base theme classes
        let (main_bg_class, main_text_class, base_border_class) = match self.current_theme {
            Theme::Light => ("bg-white", "text-gray-800", "border-gray-300"),
            Theme::Dark => ("bg-gray-800", "text-gray-100", "border-gray-600"),
        };

        // Specific themed classes (some might reuse base_border_class or define their own)
        let panel_bg_color = if self.current_theme == Theme::Light { "bg-gray-100" } else { "bg-gray-700" };
        let item_bg_color = if self.current_theme == Theme::Light { "bg-white" } else { "bg-gray-600" };
        let input_bg_color = if self.current_theme == Theme::Light { "bg-gray-100" } else { "bg-gray-700" };
        let input_text_color = if self.current_theme == Theme::Light { "focus:text-gray-700" } else { "text-gray-100 placeholder-gray-400 focus:text-gray-100" };
        let emoji_button_bg = if self.current_theme == Theme::Light { "bg-gray-200" } else { "bg-gray-600 hover:bg-gray-500" };
        let emoji_picker_bg = if self.current_theme == Theme::Light { "bg-white border-gray-300" } else { "bg-gray-700 border-gray-600" }; // Uses its own border or could use base_border_class
        let emoji_picker_item_hover_bg = if self.current_theme == Theme::Light { "hover:bg-gray-100" } else { "hover:bg-gray-600" };
        // Use base_border_class for consistent border colors where needed, or define specific ones
        let border_color_class = base_border_class; 
        
        html! {
            <div class={classes!("flex", "w-screen", main_bg_class, main_text_class)}>
                <div class={classes!("flex-none", "w-56", "h-screen", panel_bg_color)}>
                    <div class={classes!("text-xl", "p-3", main_text_class)}>
                        {"Users"}
                        <button onclick={toggle_theme.clone()} class={classes!("ml-4", "p-1", "text-sm", "border", border_color_class, "rounded")}>
                            { if self.current_theme == Theme::Light { "Dark Mode" } else { "Light Mode" } }
                        </button>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class={classes!("flex", "m-3", item_bg_color, "rounded-lg", "p-2")}>
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class={classes!("flex", "text-xs", "justify-between", if self.current_theme == Theme::Dark { "text-gray-100"} else {main_text_class})}>
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class={classes!("text-xs", if self.current_theme == Theme::Dark { "text-gray-300"} else {"text-gray-400"})}>
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class={classes!("w-full", "h-14", "border-b-2", border_color_class)}>
                        <div class={classes!("text-xl", "p-3", main_text_class)}>{"💬 Chat!"}</div>
                    </div>
                    <div class={classes!("w-full", "grow", "overflow-auto", "border-b-2", border_color_class)}>
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                let message_bubble_bg = if self.current_theme == Theme::Light { "bg-gray-100" } else { "bg-gray-700" };
                                html!{
                                    <div class={classes!("flex", "items-end", "w-3/6", message_bubble_bg, "m-8", "rounded-tl-lg", "rounded-tr-lg", "rounded-br-lg")}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class={classes!("text-sm", if self.current_theme == Theme::Dark { "text-gray-100"} else {main_text_class})}>
                                                {m.from.clone()}
                                            </div>
                                            <div class={classes!("text-xs", if self.current_theme == Theme::Dark { "text-gray-300"} else {"text-gray-500"})}>
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }

                    </div>
                    <div class="w-full h-14 flex px-3 items-center relative">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class={classes!("block", "w-full", "py-2", "pl-4", "mx-3", input_bg_color, "rounded-full", "outline-none", input_text_color, border_color_class, "border")} name="message" required=true />
                        
                        <button onclick={toggle_emoji_picker} class={classes!("p-2", "mr-2", "shadow-sm", emoji_button_bg, "w-10", "h-10", "rounded-full", "flex", "justify-center", "items-center", if self.current_theme == Theme::Dark { "text-gray-100" } else { main_text_class } )}>
                            {"😊"}
                        </button>
                        
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center color-white">
                            <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                        
                        // Emoji picker
                        {
                            if self.show_emoji_picker {
                                html! {
                                    <div class={classes!("absolute", "bottom-16", "right-16", emoji_picker_bg, "p-2", "rounded-lg", "shadow-lg", "border", "grid", "grid-cols-4", "gap-2", "z-10")}> // emoji_picker_bg includes border
                                        {
                                            emojis.iter().map(|emoji| {
                                                let emoji_clone = emoji.to_string();
                                                let select_emoji = ctx.link().callback(move |_| Msg::SelectEmoji(emoji_clone.clone()));
                                                
                                                html! {
                                                    <button onclick={select_emoji} class={classes!("text-2xl", "p-2", emoji_picker_item_hover_bg, "rounded", "cursor-pointer", if self.current_theme == Theme::Dark { "text-gray-100"} else {main_text_class})}>
                                                        {emoji}
                                                    </button>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </div>
        }
    }
}
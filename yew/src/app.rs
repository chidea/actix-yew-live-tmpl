use log::*;
use serde_derive::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use stdweb::{_js_impl, js};
use yew::{
  events::IKeyboardEvent,
  format::{Json, Text, Binary},
  services::{
    storage::{Area, StorageService}, 
    websocket::{WebSocketService, WebSocketStatus, WebSocketTask},
    // FetchService,
    timeout::{TimeoutService, TimeoutTask},
    Task,
  },
  html, Component, ComponentLink, Href, Html, Renderable, ShouldRender
};
use std::time::Duration;
use failure::Error;

const KEY: &'static str = "yew.todomvc.self";

pub struct App {
  storage: StorageService,
  wss: WebSocketService,
  timeout: TimeoutService,
  link: ComponentLink<App>,
  state: State,
  ws: Option<WebSocketTask>,
  reloadTask:Option<TimeoutTask>,
}

#[derive(Serialize, Deserialize)]
pub struct State {
  // data: u32,
  entries: Vec<Entry>,
  filter: Filter,
  value: String,
  edit_value: String,
  reload: bool,
}

#[derive(Serialize, Deserialize)]
struct Entry {
  description: String,
  completed: bool,
  editing: bool,
}

type AsBinary = bool;

pub enum WsAction {
  Connect,
  Connected,
  SendData(Vec<u8>),
  SendText(String),
  Disconnect,
  Lost,
}

pub enum Msg {
  Init,
  Add,
  Edit(usize),
  Update(String),
  UpdateEdit(String),
  Remove(usize),
  SetFilter(Filter),
  ToggleAll,
  ToggleEdit(usize),
  Toggle(usize),
  ClearCompleted,
  WsAction(WsAction),
  WsReady(Result<String, Error>),
  // Reload,
  Nope,
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}


// /// This type is used as a request which sent to websocket connection.
// #[derive(Serialize, Debug)]
// struct WsRequest {
//     value: u32,
// }

// /// This type is an expected response from a websocket connection.
// #[derive(Deserialize, Debug)]
// pub struct WsResponse {
//     value: u32,
// }

fn get_ws_url() -> String {
  (js! { return (window.location.protocol=="https:"&&"wss://"||"ws://")+window.location.host + "/ws" }).into_string().unwrap()
}

impl Component for App {
  type Message = Msg;
  type Properties = ();

  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    let storage = StorageService::new(Area::Local);
    let entries = {
      if let Json(Ok(restored_entries)) = storage.restore(KEY) {
        restored_entries
      } else {
        Vec::new()
      }
    };
    let state = State {
      entries,
      filter: Filter::All,
      value: "".into(),
      edit_value: "".into(),
      reload: false,
    };
    let wss = WebSocketService::new();
    let timeout = TimeoutService::new();
    App { storage, wss, timeout, ws:None, state, link, reloadTask:None }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::Init => {
        info!("initiating");
        self.link.send_self(WsAction::Connect.into() );
        return false;
      }
      Msg::Add => {
        let entry = Entry {
          description: self.state.value.clone(),
          completed: false,
          editing: false,
        };
        self.state.entries.push(entry);
        self.state.value = "".to_string();
      }
      Msg::Edit(idx) => {
        let edit_value = self.state.edit_value.clone();
        self.state.complete_edit(idx, edit_value);
        self.state.edit_value = "".to_string();
      }
      Msg::Update(val) => {
        println!("Input: {}", val);
        self.state.value = val;
      }
      Msg::UpdateEdit(val) => {
        println!("Input: {}", val);
        self.state.edit_value = val;
      }
      Msg::Remove(idx) => {
        self.state.remove(idx);
      }
      Msg::SetFilter(filter) => {
        self.state.filter = filter;
      }
      Msg::ToggleEdit(idx) => {
        self.state.edit_value = self.state.entries[idx].description.clone();
        self.state.toggle_edit(idx);
      }
      Msg::ToggleAll => {
        let status = !self.state.is_all_completed();
        self.state.toggle_all(status);
      }
      Msg::Toggle(idx) => {
        self.state.toggle(idx);
      }
      Msg::ClearCompleted => {
        self.state.clear_completed();
      }
      Msg::WsAction(action) => {
        match action {
          WsAction::Connect => {
            let callback = self.link.send_back(|Json(text)| Msg::WsReady(text));
            let notification = self.link.send_back(|status| match status {
                WebSocketStatus::Opened => WsAction::Connected.into(),
                WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
            });
            let task =
                self.wss
                    .connect(get_ws_url().as_str(), callback, notification);
            info!("connecting websocket...");
            self.ws = Some(task);
            return true; // enabled websocket action buttons
          }
          WsAction::Connected => {
            info!("websocket is connected");
            if self.state.reload {
              js! { window.location.reload(); };
            }
          }
          WsAction::SendData(binary) => {
            info!("sending websocket data");
            // let request = WsRequest { value: 321 };
            // if binary {
                // self.ws.as_mut().unwrap().send_binary(Json(&request));
            // } else {
                // self.ws.as_mut().unwrap().send(Json(&request));
            // }
            self.ws.as_mut().unwrap().send_binary(Ok(binary));
          }
          WsAction::SendText(text) => {
            info!("sending websocket text");
            self.ws.as_mut().unwrap().send(Ok(text));
          }
          WsAction::Disconnect => {
            info!("disconnecting websocket");
            self.ws.take().unwrap().cancel();
          }
          WsAction::Lost => {
            info!("websocket disconnected");
            self.ws = None;
            self.state.reload = true;
            // let set_to = || self.timeout.spawn(Duration::new(1,0), self.link.send_back(|_| WsAction::Connect.into())) ;
            let mut b = false;
            if let Some(t) = &self.reloadTask {
              if t.is_active() {
                b = true;
              }
            }
            if b {
              self.reloadTask.as_mut().unwrap().cancel();
            }
            self.reloadTask = Some(self.timeout.spawn(Duration::new(1,0), self.link.send_back(|_| WsAction::Connect.into())));
          }
        }
        return false;
      }
      Msg::WsReady(response) => {
        // self.data = response.map(|data| data.value).ok();
        if let Ok(v) = response{
          info!("received : {}", v);
        }
        return true;
      }
      // Msg::Reload => {
      //   js! { window.location.reload(); };
      //   return false;
      // }
      Msg::Nope => {return false;}
    }
    self.storage.store(KEY, Json(&self.state.entries));
    true
  }
}

impl Renderable<App> for App {
  fn view(&self) -> Html<Self> {
    info!("rendered!");
    html! {
      <div class="todomvc-wrapper">
        <section class="todoapp">
          <header class="header">
            <h1>{ "Todo" }</h1>
            { self.view_input() }
          </header>
          <section class="main">
            <input class="toggle-all" type="checkbox" checked=self.state.is_all_completed() onclick=|_| Msg::ToggleAll />
            <ul class="todo-list">
              { for self.state.entries.iter().filter(|e| self.state.filter.fit(e)).enumerate().map(view_entry) }
            </ul>
          </section>
          <footer class="footer">
            <span class="todo-count">
              <strong>{ self.state.total() }</strong>
              { " item(s) left" }
            </span>
            <ul class="filters">
              { for Filter::iter().map(|flt| self.view_filter(flt)) }
            </ul>
            <button class="clear-completed" onclick=|_| Msg::ClearCompleted>
              { format!("Clear completed ({})", self.state.total_completed()) }
            </button>
          </footer>
        </section>
        <footer class="info">
          <p>{ "Double-click to edit a todo" }</p>
          <p>{ "Written by " }<a href="https://github.com/DenisKolodin/" target="_blank">{ "Denis Kolodin" }</a></p>
          <p>{ "Part of " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
        </footer>
        /* <div>
          <nav class="menu">
            /* <button onclick=|_| Msg::FetchData(Format::Json, false)>{ "Fetch Data" }</button>
            <button onclick=|_| Msg::FetchData(Format::Json, true)>{ "Fetch Data [binary]" }</button>
            <button onclick=|_| Msg::FetchData(Format::Toml, false)>{ "Fetch Data [toml]" }</button> */
            // { self.view_data() }
            <button disabled=self.ws.is_some()
                onclick=|_| WsAction::Connect.into()>{ "Connect To WebSocket" }</button>
            <button disabled=self.ws.is_none()
                onclick=|_| WsAction::SendData(false).into()>{ "Send To WebSocket" }</button>
            <button disabled=self.ws.is_none()
                onclick=|_| WsAction::SendData(true).into()>{ "Send To WebSocket [binary]" }</button>
            <button disabled=self.ws.is_none()
                onclick=|_| WsAction::Disconnect.into()>{ "Close WebSocket connection" }</button>
          </nav>
        </div> */
        <button class="ws" disabled=self.ws.is_none()
          onclick=|_| WsAction::SendText("restart_all".to_owned()).into()>{ "Refresh every instants" }</button>
      </div>
    }
  }
}

impl App {
  fn view_filter(&self, filter: Filter) -> Html<App> {
    let flt = filter.clone();
    html! {
      <li>
        <a class=if self.state.filter == flt { "selected" } else { "not-selected" }
          href=&flt
          onclick=|_| Msg::SetFilter(flt.clone())>
          { filter }
        </a>
      </li>
    }
  }

  fn view_input(&self) -> Html<App> {
    html! {
      // You can use standard Rust comments. One line:
      // <li></li>
      <input class="new-todo"
        placeholder="What needs to be done?"
        value=&self.state.value
        oninput=|e| Msg::Update(e.value)
        onkeypress=|e| {
          if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
        } />
      /* Or multiline:
      <ul>
        <li></li>
      </ul>
      */
    }
  }
  // fn view_data(&self) -> Html<App> {
  //   if let Some(value) = self.data {
  //     html! {
  //       <p>{ value }</p>
  //     }
  //   } else {
  //     html! {
  //       <p>{ "Data hasn't fetched yet." }</p>
  //     }
  //   }
  // }
}

fn view_entry((idx, entry): (usize, &Entry)) -> Html<App> {
  let mut class = "todo".to_string();
  if entry.editing {
    class.push_str(" editing");
  }
  if entry.completed {
    class.push_str(" completed");
  }
  html! {
    <li class=class>
      <div class="view">
        <input class="toggle" type="checkbox" checked=entry.completed onclick=|_| Msg::Toggle(idx) />
        <label ondoubleclick=|_| Msg::ToggleEdit(idx)>{ &entry.description }</label>
        <button class="destroy" onclick=|_| Msg::Remove(idx) />
      </div>
      { view_entry_edit_input((idx, &entry)) }
    </li>
  }
}

fn view_entry_edit_input((idx, entry): (usize, &Entry)) -> Html<App> {
  if entry.editing == true {
    html! {
      <input class="edit"
          type="text"
          value=&entry.description
          oninput=|e| Msg::UpdateEdit(e.value)
          onblur=|_| Msg::Edit(idx)
          onkeypress=|e| {
            if e.key() == "Enter" { Msg::Edit(idx) } else { Msg::Nope }
          } />
    }
  } else {
    html! { <input type="hidden" /> }
  }
}

#[derive(EnumIter, ToString, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
  All,
  Active,
  Completed,
}

impl<'a> Into<Href> for &'a Filter {
  fn into(self) -> Href {
    match *self {
      Filter::All => "#/".into(),
      Filter::Active => "#/active".into(),
      Filter::Completed => "#/completed".into(),
    }
  }
}

impl Filter {
  fn fit(&self, entry: &Entry) -> bool {
    match *self {
      Filter::All => true,
      Filter::Active => !entry.completed,
      Filter::Completed => entry.completed,
    }
  }
}

impl State {
  fn total(&self) -> usize {
    self.entries.len()
  }

  fn total_completed(&self) -> usize {
    self.entries
      .iter()
      .filter(|e| Filter::Completed.fit(e))
      .count()
  }

  fn is_all_completed(&self) -> bool {
    let mut filtered_iter = self
      .entries
      .iter()
      .filter(|e| self.filter.fit(e))
      .peekable();

    if filtered_iter.peek().is_none() {
      return false;
    }

    filtered_iter.all(|e| e.completed)
  }

  fn toggle_all(&mut self, value: bool) {
    for entry in self.entries.iter_mut() {
      if self.filter.fit(entry) {
        entry.completed = value;
      }
    }
  }

  fn clear_completed(&mut self) {
    let entries = self
      .entries
      .drain(..)
      .filter(|e| Filter::Active.fit(e))
      .collect();
    self.entries = entries;
  }

  fn toggle(&mut self, idx: usize) {
    let filter = self.filter.clone();
    let mut entries = self
      .entries
      .iter_mut()
      .filter(|e| filter.fit(e))
      .collect::<Vec<_>>();
    let entry = entries.get_mut(idx).unwrap();
    entry.completed = !entry.completed;
  }

  fn toggle_edit(&mut self, idx: usize) {
    let filter = self.filter.clone();
    let mut entries = self
      .entries
      .iter_mut()
      .filter(|e| filter.fit(e))
      .collect::<Vec<_>>();
    let entry = entries.get_mut(idx).unwrap();
    entry.editing = !entry.editing;
  }

  fn complete_edit(&mut self, idx: usize, val: String) {
    let filter = self.filter.clone();
    let mut entries = self
      .entries
      .iter_mut()
      .filter(|e| filter.fit(e))
      .collect::<Vec<_>>();
    let entry = entries.get_mut(idx).unwrap();
    entry.description = val;
    entry.editing = !entry.editing;
  }

  fn remove(&mut self, idx: usize) {
    let idx = {
      let filter = self.filter.clone();
      let entries = self
        .entries
        .iter()
        .enumerate()
        .filter(|&(_, e)| filter.fit(e))
        .collect::<Vec<_>>();
      let &(idx, _) = entries.get(idx).unwrap();
      idx
    };
    self.entries.remove(idx);
  }
}


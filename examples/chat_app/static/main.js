'use strict';
import { getCurrentUser, getMessages } from "./api.js";

/**
 * @type {{id: string, username: string, content: string }[]}
 */
let messages = [];

/**
 * @type {{username:string}|null}
 */
let currentUser = null;

window.addEventListener("load", async () => {
  await initialize();
  chat();
});

const messagesListEl = document.querySelector("#messages");
const chatBoxFormEl = document.querySelector("#chatbox");
const chatMessageTemplate = document.querySelector("#chat-message").content;

function render(newMessages) {
  messages = newMessages;
  messagesListEl.replaceChildren([]);

  for (const { id, username, content } of newMessages) {
    /**
     * @type {HTMLElement}
     */
    const userMessageEl = chatMessageTemplate.cloneNode(true).querySelector("li");

    if (currentUser && currentUser.username === username) {
      userMessageEl.style.fontWeight = "bold";
    }

    userMessageEl.dataList = `user/${id}`;
    userMessageEl.innerText = `${username}: ${content || "<empty>"}`;
    messagesListEl.append(userMessageEl);
  }

  if (newMessages.length === 0) {
    const li = document.createElement("li");
    li.textContent = "No messages";
    messagesListEl.append(li);
  }
}

async function initialize() {
  // Initialize chat messages
  const initialMessages = await getMessages();
  currentUser = await getCurrentUser();
  messages = [...initialMessages, ...messages];
  render(messages);
}

function chat() {
  const abortController = new AbortController();
  const ws = new WebSocket("/api/chat");

  ws.onopen = () => {
    console.log("Chat started...");
  };

  ws.onmessage = (ev) => {
    console.log("Chat message received: ", ev.data);
    const msg = JSON.parse(ev.data);
    render([...messages, msg]);
  };

  ws.onclose = () => {
    console.log("Chat closed");

    // Try reconnect
    // setTimeout(() => {
    //   console.log("Reconnecting...");
    //   abortController.abort();
    //   chat();
    // }, 1000);
  };

  /**
   * @param {Event} ev
   */
  function handleChatMessageSubmit(ev) {
    ev.preventDefault();

    const form = new FormData(ev.currentTarget);
    const content = form.get("content");
    const id = `local_${crypto.randomUUID()}`;
    const { username } = currentUser;
    const newMessage = { id, username, content };
    render([...messages, newMessage]);

    // Send chat message
    ws.send(content);
    ev.currentTarget.reset();
  }

  chatBoxFormEl.addEventListener("submit", handleChatMessageSubmit, {
    signal: abortController.signal,
  });
}

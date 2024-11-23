"use strict";

const messagesListEl = document.querySelector("#messages");
const chatBoxFormEl = document.querySelector("#chatbox-form");
const chatBoxTextAreaEl = document.querySelector("#chatbox-textarea");
const chatMessageTemplate = document.querySelector("#chat-message").content;

/**
 * @type {{id: string, username: string, content: string }[]}
 */
let messages = [];

/**
 * @type {{username:string}|null}
 */
let currentUser = null;

/**
 * Get all the chat messages
 * @returns {Promise<{id: string, username: string, content: string}[]>}
 */
async function getMessages() {
  const res = await fetch("/api/chat/messages");
  return await res.json();
}

/**
 * Get the current user
 * @returns {Promise<{username: string }>}
 */
async function getCurrentUser() {
  const res = await fetch("/api/me");
  return await res.json();
}

function updateChat(newMessages) {
  messages = newMessages;
  messagesListEl.replaceChildren([]);

  for (const { id, username, content } of newMessages) {
    /**
     * @type {HTMLElement}
     */
    const userMessageEl = chatMessageTemplate
      .cloneNode(true)
      .querySelector("li");

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

window.addEventListener("load", async () => {
  // Fetch chat messages
  const initialMessages = await getMessages();
  currentUser = await getCurrentUser();
  messages = [...initialMessages, ...messages];
  updateChat(messages);

  // Initialize web socket
  const ws = new WebSocket("/api/chat");

  ws.onopen = () => {
    console.log("Chat started...");
  };

  ws.onmessage = (ev) => {
    console.log("Chat message received: ", ev.data);
    const msg = JSON.parse(ev.data);
    updateChat([...messages, msg]);
  };

  ws.onclose = () => {
    console.log("Chat closed");
  };

  // Listen for form submits
  chatBoxFormEl.addEventListener("submit", (ev) => {
    ev.preventDefault();

    const form = new FormData(ev.currentTarget);
    const content = form.get("content");
    const id = `local_${crypto.randomUUID()}`;
    const { username } = currentUser;
    const newMessage = { id, username, content };
    updateChat([...messages, newMessage]);

    // Send chat message
    ws.send(content);
    ev.currentTarget.reset();
  });

  // Submit on enter
  chatBoxTextAreaEl.addEventListener("keydown", (ev) => {
    if (ev.key !== "Enter") {
      return;
    }

    ev.preventDefault();
    chatBoxFormEl.submit();
  });
});

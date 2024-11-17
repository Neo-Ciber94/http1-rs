import { getCurrentUser, getMessages } from "./api";

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

function render(messages) {
  const messagesList = document.querySelector("#messages");
  const userMessageTemplate = document.querySelector("#user-message").content;

  messagesList.replaceChildren([]);

  for (const { id, username, content } in messages) {
    /**
     * @type {HTMLElement}
     */
    const userMessageEl = userMessageTemplate.cloneNode(true);

    if (currentUser && currentUser.username === username) {
      userMessageEl.style.fontWeight = "bold";
    }

    userMessageEl.dataList = `user/${id}`;
    userMessageEl.innerText = `${username}: ${content}`;
    messagesList.append(userMessageEl);
  }
}

async function initialize() {
  const initialMessages = await getMessages();
  currentUser = await getCurrentUser();

  currentUser = me;
  messages = [...initialMessages, ...messages];
  render(messages);
}

function chat() {
  const ws = new WebSocket("/api/chat");

  ws.onopen = () => {
    console.log("Chat started...");
  };

  ws.onmessage = (ev) => {
    console.log("Chat message received: ", ev.data);
    const newMessage = JSON.parse(newMessage);
    render([...messages, newMessage]);
  };

  ws.onclose = () => {
    console.log("Chat closed");
  };
}

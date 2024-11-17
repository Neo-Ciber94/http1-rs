/**
 * Get all the chat messages
 * @returns {Promise<{id: string, username: string, content: string}[]>}
 */
export async function getMessages() {
  const res = await fetch("/api/chat/messages");
  return await res.json();
}

/**
 * Get the current user
 * @returns {Promise<{username: string }>}
 */
export async function getCurrentUser() {
  const res = await fetch("/api/me");
  return await res.json();
}

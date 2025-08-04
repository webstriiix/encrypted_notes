import { get, set } from "idb-keyval";

// Generate or fetch AES-GCM key for a note
async function getNoteKey(note_id, owner) {
  const keyId = [note_id.toString(), owner];
  let noteKey = await get(keyId);
  if (!noteKey) {
    noteKey = await window.crypto.subtle.generateKey(
      { name: "AES-GCM", length: 256 },
      true,
      ["encrypt", "decrypt"]
    );
    await set(keyId, noteKey);
  }
  return noteKey;
}

// Encrypt data with note-specific key
export async function encryptWithNoteKey(note_id, owner, data) {
  const noteKey = await getNoteKey(note_id, owner);
  const enc = new TextEncoder();
  const iv = window.crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await window.crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    noteKey,
    enc.encode(data)
  );
  // Gabungkan IV dan ciphertext jadi satu string base64
  const result = new Uint8Array(iv.length + ciphertext.byteLength);
  result.set(iv, 0);
  result.set(new Uint8Array(ciphertext), iv.length);
  return btoa(String.fromCharCode(...result));
}

// Decrypt data with note-specific key
export async function decryptWithNoteKey(note_id, owner, encrypted) {
  const noteKey = await getNoteKey(note_id, owner);
  const data = Uint8Array.from(atob(encrypted), c => c.charCodeAt(0));
  const iv = data.slice(0, 12);
  const ciphertext = data.slice(12);
  const decrypted = await window.crypto.subtle.decrypt(
    { name: "AES-GCM", iv },
    noteKey,
    ciphertext
  );
  return new TextDecoder().decode(decrypted);
}
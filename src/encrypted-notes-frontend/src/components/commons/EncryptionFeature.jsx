import React, { useState } from "react";
import { encryptWithNoteKey, decryptWithNoteKey } from "../../utlis/encryption";

const note_id = 123n; // Gunakan BigInt untuk note_id
const owner = "user@example.com";

const EncryptionFeature = () => {
  const [plainText, setPlainText] = useState("");
  const [encrypted, setEncrypted] = useState("");
  const [decrypted, setDecrypted] = useState("");

  const handleEncrypt = async () => {
    const enc = await encryptWithNoteKey(note_id, owner, plainText);
    setEncrypted(enc);
    setDecrypted("");
  };

  const handleDecrypt = async () => {
    const dec = await decryptWithNoteKey(note_id, owner, encrypted);
    setDecrypted(dec);
  };

  return (
    <div className="max-w-xl mx-auto mt-8 p-6 bg-white rounded-lg shadow space-y-4">
      <h2 className="text-2xl font-bold mb-2 text-gray-800">Fitur Enkripsi Catatan</h2>
      <textarea
        className="w-full border border-gray-300 rounded p-2 text-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-400"
        rows={4}
        placeholder="Tulis catatan di sini..."
        value={plainText}
        onChange={e => setPlainText(e.target.value)}
      />
      <div className="flex gap-4">
        <button
          className="bg-blue-600 hover:bg-blue-700 text-white font-semibold px-4 py-2 rounded transition"
          onClick={handleEncrypt}
        >
          Encrypt
        </button>
        <button
          className="bg-green-600 hover:bg-green-700 text-white font-semibold px-4 py-2 rounded transition"
          onClick={handleDecrypt}
        >
          Decrypt
        </button>
      </div>
      <div>
        <label className="font-semibold text-gray-700">Encrypted:</label>
        <div className="bg-gray-100 p-2 rounded break-all text-xs text-gray-700 min-h-[40px]">{encrypted}</div>
      </div>
      <div>
        <label className="font-semibold text-gray-700">Decrypted:</label>
        <div className="bg-gray-100 p-2 rounded break-all text-gray-700 min-h-[40px]">{decrypted}</div>
      </div>
    </div>
  );
};

export default EncryptionFeature;
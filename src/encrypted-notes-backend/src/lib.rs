mod helpers;
mod storage;
mod types;

use candid::Principal;
use ic_cdk::api::msg_caller;
use ic_cdk::management_canister::{
    VetKDCurve, VetKDDeriveKeyArgs, VetKDDeriveKeyResult, VetKDKeyId, VetKDPublicKeyArgs,
    VetKDPublicKeyResult,
};
use ic_cdk::update;
use ic_stable_structures::Storable;

use crate::helpers::{assert_not_anonymous, get_next_id};
use crate::storage::NOTES;
use crate::types::{Note, NoteId};

const MAX_NOTE_SIZE: usize = 1024;

#[update]
fn create_note(encrypted: String) -> NoteId {
    let caller = msg_caller();
    assert_not_anonymous(&caller);
    assert!(encrypted.len() <= MAX_NOTE_SIZE, "Too large");

    let note_id = get_next_id();
    let note = Note {
        id: note_id,
        owner: caller,
        encrypted,
        shared_read: vec![],
        shared_edit: vec![],
    };

    NOTES.with_borrow_mut(|store| {
        store.insert(note_id, note);
    });

    note_id
}

#[update]
fn read_notes() -> Vec<Note> {
    let caller = msg_caller();
    assert_not_anonymous(&caller);

    NOTES.with_borrow(|store| {
        store
            .iter()
            .filter(|(_, note)| note.owner == caller || note.shared_read.contains(&caller))
            .map(|(_, note)| note.clone())
            .collect()
    })
}

#[update]
fn update_note(note_id: NoteId, new_encrypted: String) {
    let caller = msg_caller();
    assert!(new_encrypted.len() <= MAX_NOTE_SIZE);

    NOTES.with_borrow_mut(|store| {
        if let Some(mut note) = store.get(&note_id) {
            if !note.can_edit(&caller) {
                ic_cdk::trap("Not authorized to update this note");
            }

            note.encrypted = new_encrypted;
            store.insert(note_id, note);
        } else {
            ic_cdk::trap("Note not found");
        }
    });
}

#[update]
fn delete_note(note_id: NoteId) {
    let caller = msg_caller();
    NOTES.with_borrow_mut(|store| {
        if let Some(note) = store.get(&note_id) {
            if note.owner != caller {
                ic_cdk::trap("Only owner can delete");
            }
            store.remove(&note_id);
        }
    });
}

#[update]
fn share_note_read(note_id: NoteId, user: Principal) {
    let caller = msg_caller();

    NOTES.with_borrow_mut(|store| {
        if let Some(mut note) = store.get(&note_id) {
            if note.owner != caller {
                ic_cdk::trap("Only owner can share");
            }

            if !note.shared_read.contains(&user) {
                note.shared_read.push(user);
                store.insert(note_id, note);
            }
        }
    });
}

#[update]
fn share_note_edit(note_id: NoteId, user: Principal) {
    let caller = msg_caller();

    NOTES.with_borrow_mut(|store| {
        if let Some(mut note) = store.get(&note_id) {
            if note.owner != caller {
                ic_cdk::trap("Only owner can share");
            }

            if !note.shared_edit.contains(&user) {
                note.shared_edit.push(user);
                store.insert(note_id, note);
            }
        }
    });
}

#[update]
fn unshare_note_read(note_id: NoteId, user: Principal) {
    let caller = msg_caller();

    NOTES.with_borrow_mut(|store| {
        if let Some(mut note) = store.get(&note_id) {
            if note.owner != caller {
                ic_cdk::trap("Only owner can unshare");
            }
            note.shared_read.retain(|p| p != &user);
            store.insert(note_id, note);
        }
    });
}

#[update]
fn unshare_note_edit(note_id: NoteId, user: Principal) {
    let caller = msg_caller();

    NOTES.with_borrow_mut(|store| {
        if let Some(mut note) = store.get(&note_id) {
            if note.owner != caller {
                ic_cdk::trap("Only owner can unshare");
            }
            note.shared_edit.retain(|p| p != &user);
            store.insert(note_id, note);
        }
    });
}

// encryption logic
fn bls12_381_g2_test_key_1() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381_G2,
        name: "test_key_1".to_string(),
    }
}

#[update]
async fn symmetric_key_verification_key_for_note() -> String {
    let request = VetKDPublicKeyArgs {
        canister_id: None,
        context: b"note_symmetric_key".to_vec(),
        key_id: bls12_381_g2_test_key_1(),
    };

    let response: VetKDPublicKeyResult = ic_cdk::management_canister::vetkd_public_key(&request)
        .await
        .expect("call to vetkd_public_key failed");

    hex::encode(response.public_key)
}

#[update]
async fn encrypted_symmetric_key_for_note(
    note_id: NoteId,
    transport_public_key: Vec<u8>,
) -> String {
    let caller = msg_caller();

    let request = NOTES.with_borrow(|notes| {
        if let Some(note) = notes.get(&note_id) {
            if !note.can_read(&caller) {
                ic_cdk::trap(format!(
                    "unauthorized key request by user {}",
                    caller.to_text()
                ));
            }

            VetKDDeriveKeyArgs {
                input: {
                    let mut buf = vec![];
                    buf.extend_from_slice(&note_id.to_be_bytes());
                    buf.extend_from_slice(&note.owner.to_bytes());
                    buf
                },
                context: b"note_symmetric_key".to_vec(),
                key_id: bls12_381_g2_test_key_1(),
                transport_public_key,
            }
        } else {
            ic_cdk::trap(format!("note with ID {note_id} does not exist"));
        }
    });

    let response: VetKDDeriveKeyResult = ic_cdk::management_canister::vetkd_derive_key(&request)
        .await
        .expect("call to vetkd_derive_key failed");

    hex::encode(response.encrypted_key)
}

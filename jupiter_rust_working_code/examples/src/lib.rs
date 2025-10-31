use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::{deserialize, serialize};
use dotenv::dotenv;

use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::VersionedTransaction,
};
use std::env;

/// Signs a base64-encoded Solana transaction using a private key from `.env` file,
/// and returns the signed transaction re-encoded in base64 format.
///
/// # Environment Variables
/// - `PRIVATE_KEY`: A base58-encoded private key (64-byte array) stored in the `.env` file.
///
/// # Arguments
/// - `transaction` - A base64-encoded `VersionedTransaction` to be signed.
///
/// # Returns
/// - A base64-encoded, signed `VersionedTransaction`.
///
/// # Panics
/// - If the `.env` file can't be loaded.
/// - If the `PRIVATE_KEY` is not set or is invalid.
/// - If decoding/encoding fails at any step (base64, base58, bincode).
///
/// # Notes
/// - This function assumes the transaction requires a single signature.
/// - Modify the signature insertion logic if multiple signatures are needed.
///
/// # Example
/// ```
/// let signed_tx = sign_transaction(base64_tx_string);
/// ```
pub fn sign_transaction(transaction: String) -> String {
    //  Load .env file
    dotenv().ok();

    // Fetch the base58-encoded private key from the environment
    let key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set in .env");

    // Decode the private key from base58 to raw bytes. If you have raw bytes, you can skip decode
    // and just pass them directly to Keypair::from_bytes.
    let key_bytes = bs58::decode(key)
        .into_vec()
        .expect("Failed to decode base58 private key");

    // Construct a Keypair from the private key bytes
    let keypair = Keypair::from_bytes(&key_bytes).expect("Failed to create Keypair");

    // Decode the incoming base64-encoded transaction into raw bytes
    let swap_tx_bytes = STANDARD
        .decode(transaction)
        .expect("Failed to decode base64 transaction");

    // Deserialize the raw bytes into a VersionedTransaction struct
    let mut tx: VersionedTransaction = deserialize(&swap_tx_bytes).unwrap();

    // Serialize the message to be signed (excluding signatures)
    let message = tx.message.serialize();

    // sign the transaction with the keypair
    let signature = keypair.sign_message(&message);

    // If there is a need for multiple signatures (fee payer), you can modify this part accordingly.

    // Inject the signature into the transaction
    if tx.signatures.is_empty() {
        tx.signatures.push(signature);
    } else {
        tx.signatures[0] = signature;
    };

    // Serialize the signed transaction
    let signed_tx_bytes = serialize(&tx).unwrap();

    // Encode the signed transaction back to base64
    STANDARD.encode(&signed_tx_bytes)
}

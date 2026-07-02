use petstore::apis::configuration::Configuration;
use petstore::apis::pet_api;
use petstore::models::{Category, Pet};
use petstore::models::pet::Status;

/// Demonstrates using the generated petstore client.
///
/// Run with:
///   cargo run -p simple
///
/// This calls the live Petstore API at https://petstore.swagger.io/v2.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Configuration::new();
    println!("Petstore API base path: {}", cfg.base_path);

    // ── 1. Fetch a pet by ID ────────────────────────────────────────
    match pet_api::get_pet_by_id(&cfg, 1).await {
        Ok(pet) => println!("✅ Fetched pet #1: {} ({:?})", pet.name, pet.status),
        Err(e) => eprintln!("⚠️  Could not fetch pet #1 (expected if empty DB): {e}"),
    }

    // ── 2. Create a new pet ─────────────────────────────────────────
    let new_pet = Pet {
        id: None,
        name: "Rusty".to_string(),
        photo_urls: vec!["https://example.com/rusty.png".to_string()],
        category: Some(Box::new(Category {
            id: None,
            name: Some("Canine".to_string()),
        })),
        tags: None,
        status: Some(Status::Available),
    };

    match pet_api::add_pet(&cfg, new_pet).await {
        Ok(created) => println!(
            "✅ Created pet #{}: {} — status: {:?}",
            created.id.unwrap_or_default(),
            created.name,
            created.status,
        ),
        Err(e) => eprintln!("⚠️  Could not create pet: {e}"),
    }

    // ── 3. Search for available pets ────────────────────────────────
    match pet_api::find_pets_by_status(&cfg, vec!["available".to_string()]).await {
        Ok(pets) => {
            println!(
                "✅ Found {} available pet(s):",
                pets.len()
            );
            for pet in pets.iter().take(5) {
                println!("   - #{}: {}", pet.id.unwrap_or_default(), pet.name);
            }
            if pets.len() > 5 {
                println!("   ... and {} more", pets.len() - 5);
            }
        }
        Err(e) => eprintln!("⚠️  Could not search pets: {e}"),
    }

    Ok(())
}


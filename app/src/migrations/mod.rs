pub mod m2025_12;

const MIGRATIONS: &[(&str, fn() -> Result<(), anyhow::Error>)] = &[("2025_12", m2025_12::run)];

pub fn run_all() {
    println!("Starting Migrations...");
    for (name, migration_fn) in MIGRATIONS {
        match migration_fn() {
            Ok(_) => println!("Migration [{}] completed successfully.", name),
            Err(e) => eprintln!("Migration [{}] failed: {:?}", name, e),
        }
    }
    println!("Migrations complete.");
}

use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};

/// A token we assing to a user for a specific time period to prevent DDOS issues and ease
/// spam cleaning
pub struct SpamTokenGenerator {
    /// Period to update the seed
    refresh_in: Duration,
    /// A moment of the seed creation
    refreshed_at: Instant,
    /// A randomly seeded hasher
    hasher: Sha256,
}

impl SpamTokenGenerator {
    /// Creates a new generator for the specified duration
    pub fn new(refresh_in: Duration) -> Self {
        Self {
            refresh_in,
            refreshed_at: Instant::now(),
            hasher: get_rand_hasher(),
        }
    }

    /// Returns a 64-characted long token for the user
    pub fn generate(&mut self, user_id: i64) -> String {
        let mut hasher = self.hasher();
        hasher.update(user_id.to_be_bytes());
        hex::encode(hasher.finalize())
    }

    /// Updates the hasher if necessary and returns it
    fn hasher(&mut self) -> Sha256 {
        if self.refreshed_at.elapsed() > self.refresh_in {
            self.hasher = get_rand_hasher();
            self.refreshed_at = Instant::now();
        }
        self.hasher.clone()
    }
}

/// Returns a randomly seeded hasher
fn get_rand_hasher() -> Sha256 {
    let mut rng = thread_rng();
    let seed: [u8; 16] = rng.gen();
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_for_the_user() {
        let mut gen = SpamTokenGenerator::new(Duration::from_secs(60));
        assert_eq!(gen.generate(0), gen.generate(0));
    }

    #[test]
    fn differs_per_user() {
        let mut gen = SpamTokenGenerator::new(Duration::from_secs(60));
        assert_ne!(gen.generate(0), gen.generate(1));
    }

    #[test]
    fn refresh() {
        let mut gen = SpamTokenGenerator::new(Duration::ZERO);
        assert_ne!(gen.generate(0), gen.generate(0));
    }
}

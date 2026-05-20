use std::sync::{Arc, Mutex};

/// Thread-safe iteration counter for an agent.
/// 
/// Each agent (parent or subagent) gets its own `IterationBudget`.
/// The parent's budget is capped at `max_iterations`.
/// Each subagent gets an independent budget.
#[derive(Clone, Debug)]
pub struct IterationBudget {
    max_total: usize,
    used: Arc<Mutex<usize>>,
}

impl IterationBudget {
    pub fn new(max_total: usize) -> Self {
        Self {
            max_total,
            used: Arc::new(Mutex::new(0)),
        }
    }

    /// Try to consume one iteration. Returns true if allowed.
    pub fn consume(&self) -> bool {
        match self.used.lock() {
            Ok(mut used) => {
                if *used >= self.max_total {
                    false
                } else {
                    *used += 1;
                    true
                }
            }
            Err(_) => false, // Lock poisoned
        }
    }

    /// Give back one iteration.
    pub fn refund(&self) {
        if let Ok(mut used) = self.used.lock() {
            if *used > 0 {
                *used -= 1;
            }
        }
    }

    pub fn used(&self) -> usize {
        self.used.lock().map(|g| *g).unwrap_or(0)
    }

    pub fn remaining(&self) -> usize {
        let used = self.used.lock().map(|g| *g).unwrap_or(0);
        if self.max_total > used {
            self.max_total - used
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_budget() {
        let budget = IterationBudget::new(3);
        assert_eq!(budget.used(), 0);
        assert_eq!(budget.remaining(), 3);

        assert!(budget.consume());
        assert_eq!(budget.used(), 1);
        assert_eq!(budget.remaining(), 2);

        assert!(budget.consume());
        assert!(budget.consume());
        assert!(!budget.consume()); // Exhausted
        assert_eq!(budget.used(), 3);
        assert_eq!(budget.remaining(), 0);

        budget.refund();
        assert_eq!(budget.used(), 2);
        assert_eq!(budget.remaining(), 1);

        assert!(budget.consume());
        assert!(!budget.consume());
    }

    #[test]
    fn test_refund_zero() {
        let budget = IterationBudget::new(1);
        assert_eq!(budget.used(), 0);
        budget.refund();
        assert_eq!(budget.used(), 0);
    }
}

// Rust guideline compliant 2026-02-21

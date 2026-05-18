use parking_lot::RwLock;
use std::sync::Arc;

pub type SharedDraft<T> = Arc<T>;
type DraftInner<T> = (SharedDraft<T>, Option<SharedDraft<T>>);

/// Dual-state configuration manager: committed + optional draft.
/// Both committed and draft are stored as Arc<T> for zero-copy reads.
#[derive(Debug)]
pub struct Draft<T> {
    inner: Arc<RwLock<DraftInner<T>>>,
}

impl<T: Clone> Draft<T> {
    #[inline]
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new((Arc::new(data), None))),
        }
    }

    /// Get a snapshot of the committed data (zero-copy via Arc clone).
    #[inline]
    pub fn data_arc(&self) -> SharedDraft<T> {
        let guard = self.inner.read();
        Arc::clone(&guard.0)
    }

    /// Get the latest data (draft if exists, otherwise committed).
    #[inline]
    pub fn latest_arc(&self) -> SharedDraft<T> {
        let guard = self.inner.read();
        guard.1.clone().unwrap_or_else(|| Arc::clone(&guard.0))
    }

    /// Edit the draft with a closure. Uses lazy copy-on-write via Arc::make_mut.
    #[inline]
    pub fn edit_draft<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write();
        let mut draft_arc = guard.1.take().unwrap_or_else(|| Arc::clone(&guard.0));
        let data_mut = Arc::make_mut(&mut draft_arc);
        let result = f(data_mut);
        guard.1 = Some(draft_arc);
        result
    }

    /// Commit the draft, replacing the committed data.
    #[inline]
    pub fn apply(&self) {
        let mut guard = self.inner.write();
        if let Some(d) = guard.1.take() {
            guard.0 = d;
        }
    }

    /// Discard the draft (if any).
    #[inline]
    pub fn discard(&self) {
        let mut guard = self.inner.write();
        guard.1 = None;
    }

    /// Modify committed data asynchronously with optimistic locking.
    #[inline]
    pub async fn with_data_modify<F, Fut, R>(&self, f: F) -> Result<R, anyhow::Error>
    where
        T: Send + Sync + 'static,
        F: FnOnce(T) -> Fut + Send,
        Fut: std::future::Future<Output = Result<(T, R), anyhow::Error>> + Send,
    {
        let (local, original_arc) = {
            let guard = self.inner.read();
            let arc = Arc::clone(&guard.0);
            ((*arc).clone(), arc)
        };
        let (new_local, res) = f(local).await?;
        let mut guard = self.inner.write();
        if !Arc::ptr_eq(&guard.0, &original_arc) {
            return Err(anyhow::anyhow!(
                "Optimistic lock failed: Committed data has changed during async operation"
            ));
        }
        guard.0 = Arc::from(new_local);
        Ok(res)
    }
}

impl<T: Clone> Clone for Draft<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

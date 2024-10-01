//! Nautilus grammar mutator, see <https://github.com/nautilus-fuzz/nautilus>
use alloc::{borrow::Cow, string::String};
use core::{fmt::Debug, marker::PhantomData};
use std::fs::create_dir_all;

use libafl_bolts::Named;
use serde::{Deserialize, Serialize};

use crate::{
    common::nautilus::grammartec::{chunkstore::ChunkStore, context::Context},
    corpus::{Corpus, Testcase},
    events::EventFirer,
    executors::ExitKind,
    feedbacks::Feedback,
    generators::NautilusContext,
    inputs::NautilusInput,
    observers::ObserversTuple,
    state::{HasCorpus, State},
    Error, HasMetadata,
};

/// Metadata for Nautilus grammar mutator chunks
#[derive(Serialize, Deserialize)]
pub struct NautilusChunksMetadata {
    /// the chunk store
    pub cks: ChunkStore,
}

impl Debug for NautilusChunksMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NautilusChunksMetadata {{ {} }}",
            serde_json::to_string_pretty(self).unwrap(),
        )
    }
}

libafl_bolts::impl_serdeany!(NautilusChunksMetadata);

impl NautilusChunksMetadata {
    /// Creates a new [`NautilusChunksMetadata`]
    #[must_use]
    pub fn new(work_dir: String) -> Self {
        create_dir_all(format!("{}/outputs/chunks", &work_dir))
            .expect("Could not create folder in workdir");
        Self {
            cks: ChunkStore::new(work_dir),
        }
    }
}

/// A nautilus feedback for grammar fuzzing
pub struct NautilusFeedback<'a, S> {
    ctx: &'a Context,
    phantom: PhantomData<S>,
}

impl<S> Debug for NautilusFeedback<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NautilusFeedback {{}}")
    }
}

impl<'a, S> NautilusFeedback<'a, S> {
    /// Create a new [`NautilusFeedback`]
    #[must_use]
    pub fn new(context: &'a NautilusContext) -> Self {
        Self {
            ctx: &context.ctx,
            phantom: PhantomData,
        }
    }
}

impl<'a, S> Named for NautilusFeedback<'a, S> {
    fn name(&self) -> &Cow<'static, str> {
        static NAME: Cow<'static, str> = Cow::Borrowed("NautilusFeedback");
        &NAME
    }
}

impl<'a, S> Feedback<S> for NautilusFeedback<'a, S>
where
    S: HasMetadata + HasCorpus + State<Input = NautilusInput>,
    S::Corpus: Corpus<Input = NautilusInput>,
{
    #[allow(clippy::wrong_self_convention)]
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        _input: &NautilusInput,
        _observers: &OT,
        _exit_kind: &ExitKind,
    ) -> Result<bool, Error>
    where
        EM: EventFirer<State = S>,
        OT: ObserversTuple<NautilusInput, S>,
    {
        Ok(false)
    }

    fn append_metadata<EM, OT>(
        &mut self,
        state: &mut S,
        _manager: &mut EM,
        _observers: &OT,
        testcase: &mut Testcase<S::Input>,
    ) -> Result<(), Error>
    where
        OT: ObserversTuple<NautilusInput, S>,
    {
        state.corpus().load_input_into(testcase)?;
        let input = testcase.input().as_ref().unwrap().clone();
        let meta = state
            .metadata_map_mut()
            .get_mut::<NautilusChunksMetadata>()
            .expect("NautilusChunksMetadata not in the state");
        meta.cks.add_tree(input.tree, self.ctx);
        Ok(())
    }

    fn discard_metadata(&mut self, _state: &mut S, _input: &NautilusInput) -> Result<(), Error> {
        Ok(())
    }
    #[cfg(feature = "track_hit_feedbacks")]
    fn last_result(&self) -> Result<bool, Error> {
        Ok(false)
    }
}

use swc_next_ecma_ast::NodeId;

/// Node ids enclosing the current traversal position, outermost first.
///
/// `enter_node` and `exit_node` push and pop once per visited node, so the
/// stack keeps raw `top` and `end` pointers into its buffer: a push is one
/// bounds check and a store, and a pop is one pointer decrement.
pub(crate) struct NodePath {
    /// Owns the allocation. Its length is not maintained; `top` marks the end
    /// of the initialized prefix.
    buf: Vec<NodeId>,
    top: *mut NodeId,
    end: *mut NodeId,
}

impl NodePath {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let mut buf: Vec<NodeId> = Vec::with_capacity(capacity.max(1));
        let top = buf.as_mut_ptr();
        // SAFETY: `capacity` elements fit in the allocation that starts at `top`.
        let end = unsafe { top.add(buf.capacity()) };
        Self { buf, top, end }
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, node: NodeId) {
        if self.top == self.end {
            self.grow();
        }
        // SAFETY: `top < end`, so `top` is in bounds of the allocation.
        unsafe {
            self.top.write(node);
            self.top = self.top.add(1);
        }
    }

    /// Pop `node`, the most recently pushed id.
    ///
    /// # Safety
    ///
    /// The stack must be non-empty.
    #[inline(always)]
    pub(crate) unsafe fn pop(&mut self, node: NodeId) {
        debug_assert_eq!(self.as_slice().last(), Some(&node));
        // SAFETY: the caller guarantees an element below `top`.
        self.top = unsafe { self.top.sub(1) };
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self) {
        let len = self.len();
        // SAFETY: the first `len` elements are initialized and within capacity.
        unsafe { self.buf.set_len(len) };
        self.buf.reserve(1);
        let base = self.buf.as_mut_ptr();
        // SAFETY: `len` and `capacity` are in bounds of the new allocation.
        unsafe {
            self.top = base.add(len);
            self.end = base.add(self.buf.capacity());
        }
    }

    #[inline]
    fn len(&self) -> usize {
        // SAFETY: `top` points into or one past the allocation that starts at `buf`.
        unsafe { self.top.offset_from(self.buf.as_ptr()) as usize }
    }

    pub(crate) fn as_slice(&self) -> &[NodeId] {
        // SAFETY: the first `len` elements are initialized.
        unsafe { std::slice::from_raw_parts(self.buf.as_ptr(), self.len()) }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

use crate::Color;
use crate::pgn::move_data::PgnMoveData;
use crate::pgn::move_tree_node::MoveTreeNode;
use crate::pgn::position_context::PgnPositionContext;
use crate::position::Position;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(crate) enum PgnPositionContextDyn<const N: usize> {
    White(PgnPositionContext<N, { Color::White }, { Color::Black }>),
    Black(PgnPositionContext<N, { Color::Black }, { Color::White }>),
}

#[derive(Clone)]
pub(crate) struct PgnBufferedPositionContext<const N: usize, const STM: Color, const OPP: Color> {
    pub(crate) current: PgnPositionContext<N, STM, OPP>,
    pub(crate) previous: Option<PgnPositionContextDyn<N>>,
}

impl<const N: usize> PgnBufferedPositionContext<N, { Color::White }, { Color::Black }> {
    pub(crate) fn append_new_move(
        self,
        new_move_data: PgnMoveData,
        new_state: Position<N, { Color::Black }>,
    ) -> PgnBufferedPositionContextDyn<N> {
        let new_node = Rc::new(RefCell::new(MoveTreeNode::<
            N,
            { Color::Black },
            { Color::White },
        >::new(new_move_data, None)));
        self.current.node.borrow_mut().add_continuation(&new_node);
        let new_current = PgnPositionContext::<N, { Color::Black }, { Color::White }> {
            node: new_node,
            state_after_move: new_state,
        };
        PgnBufferedPositionContextDyn::Black(PgnBufferedPositionContext {
            current: new_current,
            previous: Some(PgnPositionContextDyn::White(self.current)),
        })
    }
}

impl<const N: usize> PgnBufferedPositionContext<N, { Color::Black }, { Color::White }> {
    pub(crate) fn append_new_move(
        self,
        new_move_data: PgnMoveData,
        new_state: Position<N, { Color::White }>,
    ) -> PgnBufferedPositionContextDyn<N> {
        let new_node = Rc::new(RefCell::new(MoveTreeNode::<
            N,
            { Color::White },
            { Color::Black },
        >::new(new_move_data, None)));
        self.current.node.borrow_mut().add_continuation(&new_node);
        let new_current = PgnPositionContext::<N, { Color::White }, { Color::Black }> {
            node: new_node,
            state_after_move: new_state,
        };
        PgnBufferedPositionContextDyn::White(PgnBufferedPositionContext {
            current: new_current,
            previous: Some(PgnPositionContextDyn::Black(self.current)),
        })
    }
}

#[derive(Clone)]
pub(crate) enum PgnBufferedPositionContextDyn<const N: usize> {
    White(PgnBufferedPositionContext<N, { Color::White }, { Color::Black }>),
    Black(PgnBufferedPositionContext<N, { Color::Black }, { Color::White }>),
}

impl<const N: usize> PgnBufferedPositionContextDyn<N> {
    pub(crate) fn fullmove(&self) -> u16 {
        match self {
            PgnBufferedPositionContextDyn::White(ctx) => {
                ctx.current.state_after_move.get_fullmove()
            }
            PgnBufferedPositionContextDyn::Black(ctx) => {
                ctx.current.state_after_move.get_fullmove()
            }
        }
    }

    pub(crate) fn side_to_move(&self) -> Color {
        match self {
            PgnBufferedPositionContextDyn::White(_) => Color::White,
            PgnBufferedPositionContextDyn::Black(_) => Color::Black,
        }
    }

    pub(crate) fn append_move(self, new_move_data: PgnMoveData) -> Self {
        let move_ = new_move_data.move_;
        match self {
            PgnBufferedPositionContextDyn::White(ctx) => {
                let mut next = ctx.current.state_after_move.clone();
                next.make_move(move_);
                let next = next.rebrand_stm::<{ Color::Black }>();
                ctx.append_new_move(new_move_data, next)
            }
            PgnBufferedPositionContextDyn::Black(ctx) => {
                let mut next = ctx.current.state_after_move.clone();
                next.make_move(move_);
                let next = next.rebrand_stm::<{ Color::White }>();
                ctx.append_new_move(new_move_data, next)
            }
        }
    }

    pub(crate) fn previous_as_current(&self) -> Option<Self> {
        match self {
            PgnBufferedPositionContextDyn::White(ctx) => {
                ctx.previous.clone().map(|previous| match previous {
                    PgnPositionContextDyn::White(previous) => {
                        PgnBufferedPositionContextDyn::White(PgnBufferedPositionContext {
                            current: previous,
                            previous: None,
                        })
                    }
                    PgnPositionContextDyn::Black(previous) => {
                        PgnBufferedPositionContextDyn::Black(PgnBufferedPositionContext {
                            current: previous,
                            previous: None,
                        })
                    }
                })
            }
            PgnBufferedPositionContextDyn::Black(ctx) => {
                ctx.previous.clone().map(|previous| match previous {
                    PgnPositionContextDyn::White(previous) => {
                        PgnBufferedPositionContextDyn::White(PgnBufferedPositionContext {
                            current: previous,
                            previous: None,
                        })
                    }
                    PgnPositionContextDyn::Black(previous) => {
                        PgnBufferedPositionContextDyn::Black(PgnBufferedPositionContext {
                            current: previous,
                            previous: None,
                        })
                    }
                })
            }
        }
    }
}

use crate::*;
    
impl ENode {
    // returns a lossy, normalized version of the ENode, by renaming the Slots to be deterministically ordered by their first usage.
    // shape() will later be used as a normalized ENode stored in the hashcons.
    // TODO this needs to include the perm_groups of the used eclasses (AppliedIds).
    pub fn shape(&self) -> ENode {
        let slots = self.slot_order_of_flexible();

        // maps the old slot name to the new order-based name.
        let mut slotmap = SlotMap::new();

        for x in slots {
            let n = Slot(slotmap.len());
            slotmap.insert(x, n);
        }

        self.apply_slotmap_to_flexible(&slotmap)
    }

    pub fn slot_occurences_of_flexible(&self) -> Vec<Slot> {
        let mut slotlist: Vec<Slot> = Vec::new();

        match self {
            ENode::Lam(s, r) => {
                slotlist.push(*s);
                slotlist.extend(r.m.values());
            },
            ENode::App(l, r) => {
                slotlist.extend(l.m.values());
                slotlist.extend(r.m.values());
            }
            ENode::Var(s) => {
                slotlist.push(*s);
            },
        };

        slotlist
    }

    pub fn slot_order_of_flexible(&self) -> Vec<Slot> { firsts(self.slot_occurences_of_flexible()) }
    pub fn slots_of_flexible(&self) -> HashSet<Slot> { as_set(self.slot_occurences_of_flexible()) }

    pub fn apply_slotmap_to_flexible(&self, m: &SlotMap) -> ENode {
        match self {
            ENode::Lam(x, i) => ENode::Lam(m[*x], i.apply_slotmap(&m)),
            ENode::App(i1, i2) => ENode::App(i1.apply_slotmap(&m), i2.apply_slotmap(&m)),
            ENode::Var(x) => ENode::Var(m[*x]),
        }
    }
}

use std::{fmt, collections::HashMap};



struct PReg{
    num: u32,
    ready: bool
}

impl PartialEq for PReg {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

impl fmt::Display for PReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PR#{}{}", self.num, if self.ready {"+"} else {" "}) 
    }
}

#[derive(Eq, Hash, PartialEq)]
enum VReg {
    F(u32),
    R(u32),
}

const ARCH_REGS: [VReg; 8] = [VReg::F(0), VReg::F(1), VReg::F(2), VReg::F(3),
                              VReg::R(1), VReg::R(2), VReg::R(3), VReg::R(4)];

struct Inst {
    fu: u32,
    rt: Option<VReg>,
    rs1: Option<VReg>,
    rs2: Option<VReg>,
    delay: u32
}

fn parse_reg_str(r_s: &str) -> Option<VReg> {
    let r_type: Option<char> = r_s.chars().nth(0);
    let cnt = r_s.chars().count();
    let r_s: String = r_s.chars().skip(if cnt < 1 { 0 } else { cnt - 1 }).collect();
    let r_num = r_s.parse::<u32>();

    match (r_type, r_num) {
        (Some('f'), Ok(x)) => Some(VReg::F(x)),
        (Some('r'), Ok(x)) => Some(VReg::R(x)),
        _ => None
    }
}

impl Inst {
    fn new (fu: u32, rt_s: &str, rs1_s: &str, rs2_s: &str, delay: u32) -> Self {
        let rt: Option<VReg> = parse_reg_str(rt_s);
        let rs1: Option<VReg> = parse_reg_str(rs1_s);
        let rs2: Option<VReg> = parse_reg_str(rs2_s);

        Self {
            fu,
            rt,
            rs1,
            rs2,
            delay
        }
    }
}

struct ROBEntry {
    inst_ind: usize,
    rs_ind: usize,
    T: u32, 
    Told: u32,
    S: u32,
    X: u32,
    C: u32,
    R: u32,
}

impl ROBEntry {
    fn new(inst_ind: usize, rs_ind: usize, T: u32, Told: u32) -> Self {
        Self {
            inst_ind, rs_ind, T, Told,
            S: 0,
            X: 0,
            C: 0,
            R: 0
        }
    }
}

struct ROB {
    entries: Vec<ROBEntry>,
    max_entries: usize,
    head: usize,
    tail: Option<usize>
}

impl ROB {
    fn new(rob_size: usize) -> Self {
        let entries: Vec<ROBEntry> = Vec::with_capacity(rob_size);
        Self {
            entries: entries,
            max_entries: rob_size,
            head: 0,
            tail: None, // tail starts off in an uninitialized state
        }
    }
}

impl fmt::Display for ROB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, entry) in self.entries.iter().enumerate() {
            //output += (if i == self.head.try_into().unwrap() { 'h' } else { ' ' }) + (if i == self.tail.try_into().unwrap() { 't' } else { ' ' }) + ' ';
            if i == self.head { write!(f, "h")?; } else { write!(f, " ")?; }
        
            // TODO: make else conditons all one?
            if let Some(head) = self.tail { 
                if head == i { write!(f, "t")?; } else { write!(f, " ")? }; 
            } else { write!(f, " ")? };

            write!(f, "  ")?;
            if entry.T    != 0 { write!(f, "PR#{} ", entry.T)?; } else { write!(f, "PR#   ")?; }
            if entry.Told != 0 { write!(f, "PR#{} ", entry.Told)?; } else { write!(f, "PR#   ")?; }
            if entry.S    != 0 { write!(f, "{} ", entry.S)?; } else { write!(f, "  ")?; }
            if entry.X    != 0 { write!(f, "{} ", entry.X)?; } else { write!(f, "  ")?; }
            if entry.C    != 0 { write!(f, "{} ", entry.C)?; } else { write!(f, "  ")?; }
            if entry.R    != 0 { write!(f, "{} ", entry.R)?; } else { write!(f, "  ")?; }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

struct ResStation_entry {
    fu_type: u32,
    fu_num: u32,
    rs_num: u32,
    busy: bool,
    T: Option<u32>,
    T1: Option<PReg>,
    T2: Option<PReg>,
}

impl fmt::Display for ResStation_entry {
    // TODO: whole method is kind of ass
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}",
            self.fu_type,
            if self.busy { "yes" } else { "no " },
        )?;

        if let Some(T_val) = self.T { write!(f, "PR#{} ", T_val)?; }
        else { write!(f, "    ")? };

        if let Some(T1_val) = &self.T1 { write!(f, "PR#{} ", T1_val)?; }
        else { write!(f, "    ")? };

        if let Some(T2_val) = &self.T2{ write!(f, "PR#{} ", T2_val)?; }
        else { write!(f, "    ")? };

        Ok(())
    }
}

struct OOOSim {
    rob: ROB,
    map_table: HashMap<VReg, PReg>,
    free_list: Vec<u32>,
    res_stations: Vec<ResStation_entry>,
    trace: Vec<Inst>,
    cycle: usize,
    trace_ind: usize,
}

impl OOOSim {
    fn new(trace: Vec<Inst>, num_p_regs: usize, max_rob_entries: usize, res_stations_desc: Vec<(u32, u32, u32)>) -> Self {
        let rob: ROB = ROB::new(max_rob_entries);
        let mut map_table: HashMap<VReg, PReg> = HashMap::new();
        let mut free_list: Vec<u32> = Vec::new();
        let mut res_stations: Vec<ResStation_entry> = Vec::new();
        let cycle: usize = 1;
        let trace_ind: usize = 0;

        let mut p_reg_ind = 1;

        if num_p_regs <= ARCH_REGS.len() {
            panic!("Number of phyiscal registers must be greater than virtual registers!")
        }

        for reg in ARCH_REGS {
            map_table.insert(reg, PReg { num: p_reg_ind, ready: true });
            p_reg_ind += 1;
        }

        while p_reg_ind < (num_p_regs + 1).try_into().unwrap() {
            free_list.push(p_reg_ind);
            p_reg_ind += 1;
        }

        for &(fut,fun,rsn) in res_stations_desc.iter() {
            res_stations.push(ResStation_entry { fu_type: fut, fu_num: fun, rs_num: rsn, busy: false, T: None, T1: None, T2: None })
        }

        Self { rob, map_table, free_list, res_stations, trace, cycle, trace_ind }
    }
}

fn main() {
    let trace : Vec<Inst> = vec![
        Inst::new(1, "f2", ""  , "r2", 2),
        Inst::new(2, "f0", "f2", "f3", 4),
        Inst::new(1, "r1", ""  , "r1", 2),
        Inst::new(2, "f2", "f1", "f0", 2),
        Inst::new(0, "r1", ""  , "r1", 1),
        Inst::new(0, "r2", ""  , "r2", 1),
        Inst::new(1, ""  , "f2", "r1", 2),
        Inst::new(0, "r4", "r1", "r3", 1),
    ];
    
    let res_stations_desc : Vec<(u32,u32,u32)> = vec![
        (0,0,0),
        (1,0,0),
        (1,1,0),
        (2,0,0),
        (2,1,0)
    ];
    
    let sim : OOOSim = OOOSim::new(trace, 16, 8, res_stations_desc);

    for res_station in sim.res_stations {
        println!("{}", res_station);
    }
    


    // my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    // my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    // my_rob.entries.push(ROBEntry::new(2,3, 7, 8));
    // my_rob.entries.push(ROBEntry::new(2,3, 6, 0));
    // my_rob.tail = Some(3);

    // println!("{}", my_rob);
}


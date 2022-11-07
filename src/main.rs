use std::{fmt, fs::File, collections::{HashMap, VecDeque}};
use std::io::{self, BufRead};


#[derive(Clone, Copy)]
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

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
enum VReg {
    F(u32),
    R(u32),
}

impl fmt::Display for VReg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VReg::F(N) => write!(f, "f{N}"),
            VReg::R(N) => write!(f, "r{N}")
        }
    }
}

const ARCH_REGS: [VReg; 8] = [VReg::F(0), VReg::F(1), VReg::F(2), VReg::F(3),
                              VReg::R(1), VReg::R(2), VReg::R(3), VReg::R(4)];

#[derive(Debug)]
struct Inst {
    fu: u32,
    rt: Option<VReg>,
    rs1: Option<VReg>,
    rs2: Option<VReg>,
    delay: u32
}

fn parse_reg_str(r_s: &str) -> Option<VReg> {
    // first character (indicates register type)
    let r_type: Option<char> = r_s.chars().next();
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

    fn from_trace_line(line: &str) -> Result<Self, String> {
        let mut line_split = line.split(" ");
        if line_split.clone().count() != 5 {
            return Err(String::from("Line improperly formatted."));
        }
        let fu: u32 = line_split.next().unwrap().parse::<u32>().unwrap();
        let rt: &str = line_split.next().unwrap();
        let rs1: &str = line_split.next().unwrap();
        let rs2: &str = line_split.next().unwrap();
        let delay: u32 = line_split.next().unwrap().parse::<u32>().unwrap();
        Ok(Self::new(fu, rt, rs1, rs2, delay))
    }
}

struct ROBEntry {
    inst_ind: usize,
    rs_ind: usize,
    T: Option<u32>, 
    Told: Option<u32>,
    S: u32,
    X: u32,
    C: u32,
    R: u32,
}

impl ROBEntry {
    fn new(inst_ind: usize, rs_ind: usize, T: Option<u32>, Told: Option<u32>) -> Self {
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

            // same scuffed shit
            if let Some(t) = entry.T { 
                if t != 0 { write!(f, "PR#{} ", t)?; } else { write!(f, "PR#  ")?; }; 
            } else { write!(f, "PR#  ")? };
            if let Some(told) = entry.Told { 
                if told != 0 { write!(f, "PR#{} ", told)?; } else { write!(f, "PR#  ")?; }; 
            } else { write!(f, "PR#  ")? };
            if entry.S    != 0 { write!(f, "{} ", entry.S)?; } else { write!(f, "  ")?; }
            if entry.X    != 0 { write!(f, "{} ", entry.X)?; } else { write!(f, "  ")?; }
            if entry.C    != 0 { write!(f, "{} ", entry.C)?; } else { write!(f, "  ")?; }
            if entry.R    != 0 { write!(f, "{} ", entry.R)?; } else { write!(f, "  ")?; }
            writeln!(f)?;
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
        write!(f, "{} {} ",
            self.fu_type,
            if self.busy { "yes" } else { "no " },
        )?;

        if let Some(T_val) = self.T { write!(f, "PR#{} ", T_val)?; }
        else { write!(f, "    ")? };

        if let Some(T1_val) = &self.T1 { write!(f, "{} ", T1_val)?; }
        else { write!(f, "    ")? };

        if let Some(T2_val) = &self.T2{ write!(f, "{} ", T2_val)?; }
        else { write!(f, "    ")? };

        Ok(())
    }
}

struct OOOSim {
    rob: ROB,
    map_table: HashMap<VReg, PReg>,
    free_list: VecDeque<u32>,
    res_stations: Vec<ResStation_entry>,
    trace: Vec<Inst>,
    cycle: u32,
    trace_ind: usize,
}

impl OOOSim {
    fn new(trace: Vec<Inst>, num_p_regs: usize, max_rob_entries: usize, res_stations_desc: Vec<(u32, u32, u32)>) -> Self {
        let rob: ROB = ROB::new(max_rob_entries);
        let mut map_table: HashMap<VReg, PReg> = HashMap::new();
        let mut free_list: VecDeque<u32> = VecDeque::new();
        let mut res_stations: Vec<ResStation_entry> = Vec::new();
        let cycle: u32 = 1;
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
            free_list.push_back(p_reg_ind);
            p_reg_ind += 1;
        }

        for &(fut,fun,rsn) in res_stations_desc.iter() {
            res_stations.push(ResStation_entry { fu_type: fut, fu_num: fun, rs_num: rsn, busy: false, T: None, T1: None, T2: None })
        }

        Self { rob, map_table, free_list, res_stations, trace, cycle, trace_ind }
    }

    fn Retire(&mut self) {
        // find first instruction that has not retired,
        // if it's completed, retire it, otherwise,
        // it must be out of order so we will not retire anything.
        for inst in self.rob.entries.iter_mut() {
            if inst.R == 0 {
                if inst.C != 0 {
                    inst.R = self.cycle;
                    self.rob.head += 1;
                    if let Some(told) = inst.Told {
                        if told != 0 {
                            self.free_list.push_back(told);
                        }
                    } 
                }
                return
            }
        }
    }

    fn Complete(&mut self) {
        // find first instruction that has executed, then complete it
        for inst in self.rob.entries.iter_mut() {
            if let Some(trace_inst) = self.trace.get(inst.inst_ind) {
                if trace_inst.delay == 0 && inst.C == 0 {
                    inst.C = self.cycle;

                    for preg in self.map_table.values_mut() {
                        if let Some(t) = inst.T {
                            if preg.num == t {
                                preg.ready = true;
                            }
                        }
                    }

                    for rs in self.res_stations.iter_mut() {
                        if let Some(T1_val) = &mut rs.T1 {
                            if let Some(t) = inst.T {
                                if T1_val.num == t {
                                    T1_val.ready = true;
                                }
                            }
                        }

                        if let Some(T2_val) = &mut rs.T2 {
                            if let Some(t) = inst.T {
                                if T2_val.num == t {
                                    T2_val.ready = true;
                                }
                            }
                        }
                    }

                    return
                }
            }
        }
    }

    fn Execute(&mut self) {
        let mut fu_busy: Vec<(u32,u32)> = Vec::new();
        // find all instructions waiting to execute, if the functional unit is not busy
        for inst in self.rob.entries.iter_mut() {
            let mut inst_rs = &mut self.res_stations[inst.rs_ind];
            let mut inst_t = &mut self.trace[inst.inst_ind];
            
            if (inst.S != 0) && (inst.S != self.cycle) {
                if inst.X == 0 {
                    inst.X = self.cycle;
                    inst_rs.busy = false;
                    inst_rs.T = None;
                    inst_rs.T1 = None;
                    inst_rs.T2 = None;
                }

                if inst_t.delay > 0 {
                    let fu_info: (u32,u32) = (inst_rs.fu_type, inst_rs.fu_num);
                    if !fu_busy.contains(&fu_info) {
                        inst_t.delay -= 1;
                        fu_busy.push(fu_info);
                    }
                }
            }            
        }
    }

    fn Issue(&mut self) {
        for inst in self.rob.entries.iter_mut() {
            if inst.S == 0 {
                let inst_rs = &self.res_stations[inst.rs_ind];
                match (&inst_rs.T1,&inst_rs.T2) {
                    (None,_) | (_, None)=> { inst.S = self.cycle; return }
                    (Some(T1_val),Some(T2_val)) => if (T1_val.ready && T2_val.ready) { inst.S = self.cycle; return },
                    _ => (),
                }
            }
        }
    }

    fn Dispatch(&mut self) {
        // Some fail-early style conditions

        // any more instructions to process?
        if (self.trace_ind >= self.trace.len()) {
            return
        }

        if let Some(tail_val) = self.rob.tail {
            // structural hazard: rob full, no dispatch to be done
            if (tail_val + 1) > (self.rob.max_entries + self.rob.head) {
                return
            }
        }

        let new_inst = &self.trace[self.trace_ind];

        let T: Option<u32>;
        if new_inst.rt == None {
            // if no target register, T is None (don't care)
            T = None;
        } else {
            // structural hazard: free list empty, can't get a new preg
            // only a problem if new inst has a destination register
            if self.free_list.len() == 0 {
                return;
            }
            // not empty, so grab one off the top.
            T = Some(self.free_list[0]);
        }

        

        let mut all_rss_busy = true;
        let mut rs_ind = 0;

        for (i, rs) in self.res_stations.iter_mut().enumerate() {
            if (rs.fu_type == new_inst.fu) && !rs.busy {
                all_rss_busy = false;
                rs.busy = true;
                rs.T = T;
                rs.T1 = match &new_inst.rs1 {
                    // this copies the value from the hashmap
                    Some(rs1_val) => { Some(self.map_table[rs1_val]) },
                    None => None
                };
                rs.T2 = match &new_inst.rs2 {
                    // this copies the value from the hashmap
                    Some(rs2_val) => { Some(self.map_table[rs2_val]) },
                    None => None
                };
                rs_ind = i;
                break;
            }
        }

        // structural hazard: no reservation stations available
        if all_rss_busy {
            return
        }

        let Told: Option<u32>;
        let Tnew: Option<u32>;
        match &new_inst.rt {
            Some(rt_val) => {
                self.free_list.pop_front();
                Told = Some(self.map_table[rt_val].num);
                Tnew = T;

                // we expect T to be Some(X) because it is only None when new_inst.rt is None
                // TODO: unsafe?
                self.map_table.insert(*rt_val, PReg { num: T.unwrap(), ready: false });
            }
            None => {
                Told = None;
                Tnew = None;
            }
        }


        self.rob.entries.push(ROBEntry::new(self.trace_ind, rs_ind, Tnew, Told));
        self.rob.tail = match self.rob.tail {
            Some(x) => Some(x + 1),
            None => Some(0),
        };

        self.trace_ind += 1;
        
    }

    fn Sim(&mut self, cycles: u32) {
        let cycle_start = self.cycle;
        while self.cycle < cycle_start + cycles {
            self.Retire();
            self.Complete();
            self.Execute();
            self.Issue();
            self.Dispatch();

            self.cycle += 1;
        }
        println!("{}", self.rob);
        for rs in &self.res_stations {
            println!("{}", rs);
        }
        println!("vreg | preg");
        for (k,v) in &self.map_table {
            println!("{} | {}", k, v);
        }
        println!("{:#?}", self.free_list);
    }

}

fn main() {
    // let trace : Vec<Inst> = vec![
    //     Inst::new(1, "f2", ""  , "r2", 2),
    //     Inst::new(2, "f0", "f2", "f3", 4),
    //     Inst::new(1, "f1", ""  , "r1", 2),
    //     Inst::new(2, "f2", "f1", "f0", 2),
    //     Inst::new(0, "r1", ""  , "r1", 1),
    //     Inst::new(0, "r2", ""  , "r2", 1),
    //     Inst::new(1, ""  , "f2", "r1", 2),
    //     Inst::new(0, "r4", "r1", "r3", 1),
    // ];

    let trace_iter = io::BufReader::new(File::open("r10k.trace").unwrap()).lines();
    let trace: Vec<Inst> = trace_iter
        .filter_map(Result::ok)
        .map(|x| Inst::from_trace_line(&x))
        .filter_map(Result::ok)
        .collect();
    
    let res_stations_desc : Vec<(u32,u32,u32)> = vec![
        (0,0,0),
        (1,0,0),
        (1,1,0),
        (2,0,0),
        (2,1,0)
    ];
    
    let mut sim : OOOSim = OOOSim::new(trace, 16, 8, res_stations_desc);

    sim.Sim(1000);
}


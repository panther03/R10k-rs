from dataclasses import dataclass

@dataclass
class PReg:
    reg: int
    ready: bool
    def __eq__(self, __o: object) -> bool:
        return isinstance(__o, PReg) and self.reg == __o.reg

    def __str__(self) -> str:
        return "PR#" + str(self.reg) + ("+" if self.ready else " ")

@dataclass
class Inst:
    fu: int
    rt: str
    rs1: str
    rs2: str
    delay: int

@dataclass
class ROB_entry:
    inst_ind: int
    rs_ind: int
    T: int
    Told: int
    S: int = 0
    X: int = 0
    C: int = 0
    R: int = 0

class ROB:

    def __init__(self, max_entries):
        self.entries = []
        self.max_entries = max_entries
        self.head = 0
        self.tail = -1

    def __str__(self):
        output = ""
        for i,entry in enumerate(self.entries):
            output += ('h' if i == self.head else ' ') + ('t' if i == self.tail else ' ') + '  '
            output += f"PR#{entry.T if entry.T else '  '} PR#{entry.Told if entry.Told else '  '} {entry.S if entry.S != 0 else ' '} {entry.X if entry.X != 0 else ' '} {entry.C if entry.C != 0 else ' '} {entry.R if entry.R != 0 else ' '}\n"
        return output

@dataclass
class ResStation_entry:
    fu_type: int
    fu_num: int
    rs_num: int
    busy: bool 
    T: int
    T1: PReg
    T2: PReg

    def __str__(self):
        return f"{self.fu_type} {'yes' if self.busy else 'no '} PR#{self.T} {self.T1} {self.T2}\n"

class OOOSim:
    rob: ROB
    map_table: dict[str,PReg]
    free_list: list[int]
    res_stations: list[ResStation_entry]
    trace: list[Inst]

    cycle: int
    trace_ind: int

    def __init__(self, trace, params, res_stations_desc):

        self.rob = ROB(params["max_rob_entries"])
        self.map_table = {}
        self.free_list = []
        self.res_stations = []
        self.trace = trace

        self.cycle = 1
        self.trace_ind = 0

        regs = ["f0","f1","f2","f3","r1","r2","r3","r4"]

        num_p_regs = params["num_p_regs"]
        p_reg_ind = 1
        assert(num_p_regs > len(regs))

        # fill out initial map table
        for reg in regs:
            if reg:
                self.map_table[reg] = PReg(p_reg_ind, True)
                p_reg_ind += 1

        # and free list based on remaining free physical regs
        while p_reg_ind < num_p_regs + 1:
            self.free_list.append(p_reg_ind)
            p_reg_ind += 1
        
        # fill out initial reservation stations
        # one for each reservation station as described;
        # fill out functional unit
        for (fut,fun,rsn) in res_stations_desc:
            self.res_stations.append(ResStation_entry(fu_type=fut, fu_num=fun, rs_num=rsn, \
            busy=False, T=None, T1=None, T2=None))

    def Retire(self):
        # find first instruction that has not retired,
        # if it's completed, retire it, otherwise,
        # it must be out of order so we will not retire anything.
        for inst in self.rob.entries:
            if inst.R == 0:
                if inst.C:
                    inst.R = self.cycle
                    self.rob.head += 1
                    if inst.Told:
                        self.free_list.append(inst.Told)
                return
            

    def Complete(self):
        # find first instruction that has executed,
        # then complete it.
        for inst in self.rob.entries:
            if self.trace[inst.inst_ind].delay == 0 and inst.C == 0:
                inst.C = self.cycle
                for _,preg in self.map_table.items():
                    if preg.reg == inst.T:
                        preg.ready = True
                        #self.map_table[reg] = PReg(preg.reg, True)
                for rs in self.res_stations:
                    if rs.T1 and (rs.T1.reg == inst.T):
                        rs.T1.ready = True
                    if rs.T2 and (rs.T2.reg == inst.T):
                        rs.T2.ready = True
                return
                    

    def Execute(self):
        fu_busy = []
        # find all instructions waiting to execute, if the functional unit is not busy
        for inst in self.rob.entries:
            inst_rs = self.res_stations[inst.rs_ind]
            inst_t = self.trace[inst.inst_ind]
            if inst.S and (inst.S != self.cycle):
                if (inst.X == 0): 
                    inst.X = self.cycle
                    inst_rs.busy = False
                    inst_rs.T = None
                    inst_rs.T1 = None
                    inst_rs.T2 = None
            
                if inst_t.delay > 0:
                    fu_info = (inst_rs.fu_type,inst_rs.fu_num)
                    if fu_info not in fu_busy:
                        inst_t.delay -= 1
                        fu_busy.append(fu_info)

    def Issue(self):
        for inst in self.rob.entries:
            if inst.S == 0:
                inst_rs = self.res_stations[inst.rs_ind]
                if not inst_rs.T1 or not inst_rs.T2 or (inst_rs.T1.ready and inst_rs.T2.ready):
                    inst.S = self.cycle
                    return
        
    def Dispatch(self):
        # any more instructions to process?
        if (self.trace_ind >= len(self.trace)):
            return

        # structural hazard: rob full, no dispatch to be done
        if (self.rob.tail - self.rob.head + 1) > self.rob.max_entries:
            return

        new_inst = self.trace[self.trace_ind]

        
        if new_inst.rt:
            # stuctural hazard: free list empty, can't get new preg
            # only a problem if new inst has a destination register
            if (len(self.free_list) == 0):
                return
            # not empty, so grab one off the top.
            T = self.free_list[0]
        else:
            # if no target register, T is None (don't care)
            T = None
            
    
        all_rss_busy = True
        rs_ind = 0
        for i, rs in enumerate(self.res_stations):
            if (rs.fu_type == new_inst.fu) and not rs.busy:
                all_rss_busy = False
                rs.busy = True
                rs.T = T
                rs.T1 = self.map_table[new_inst.rs1] if new_inst.rs1 else None
                rs.T2 = self.map_table[new_inst.rs2] if new_inst.rs2 else None
                rs_ind = i
                break
        
        # structural hazard: no reservation stations available
        if all_rss_busy:
            return

        if new_inst.rt:
            self.free_list.pop(0)
            Told = self.map_table[new_inst.rt].reg
        else:
            T = None
            Told = None

        self.rob.entries.append(ROB_entry(self.trace_ind, rs_ind, T, Told))
        self.rob.tail += 1

        self.map_table[new_inst.rt] = PReg(T, False)

        self.trace_ind += 1


    def Sim(self, cycles):
        cycle_start = self.cycle
        while self.cycle < cycle_start + cycles:
            self.Retire()
            self.Complete()
            self.Execute()
            self.Issue()
            self.Dispatch()
            
            self.cycle += 1
        #print(self.rob)
        #for rs in self.res_stations:
#            print(rs)
#        print(self.map_table)
#        print(self.free_list)

def parse_inst(inst_line: str) -> Inst:
    inst_line_l = inst_line.split(" ")
    assert(len(inst_line_l) == 5)
    fu = int(inst_line_l[0])
    rt = inst_line_l[1] if inst_line_l[1] != "X" else ""
    rs1 = inst_line_l[2] if inst_line_l[2] != "X" else ""
    rs2 = inst_line_l[3] if inst_line_l[3] != "X" else ""
    delay = int(inst_line_l[4])
    return Inst(fu, rt, rs1, rs2, delay)



if __name__ == "__main__":
    params = {"max_rob_entries": 8,
              "num_p_regs": 16}
    res_stations_desc = [(0,0,0),(1,0,0),(1,1,0),(2,0,0),(2,1,0)]
    """ 
    trace = [Inst(1,"f2","","r2",2),
             Inst(2,"f0","f2","f3",4),
             Inst(1,"f1","","r1",2),
             Inst(2,"f2","f1","f0",2),
             Inst(0,"r1","","r1",1),
             Inst(0,"r2","","r2",1),
             Inst(1,"","f2","r1",2),
             Inst(0,"r4","r1","r3",1)]
    """

    trace = []

    with open("r10k.trace") as trace_f:
        for line in trace_f:
            trace.append(parse_inst(line))
            
    sim = OOOSim(trace, params, res_stations_desc)
    sim.Sim(1000)


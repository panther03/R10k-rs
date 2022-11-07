import random

TRACE_DEFAULT_NAME = "r10k.trace"

TRACE_NUM_INSNS = 1000

ARCH_REGS = ["f0","f1","f2","f3","r1","r2","r3","r4"]

if __name__ == "__main__":
    # X = no register
    with open(TRACE_DEFAULT_NAME, 'w') as f:
        for _ in range(TRACE_NUM_INSNS):
            f.write(str(random.randint(0,2)))
            f.write(" ")
            # roughly 20% stores
            if random.randint(0,10) >= 8: 
                f.write("X")
                f.write(" ")
                f.write(ARCH_REGS[random.randrange(0,len(ARCH_REGS))])
                f.write(" ")
                f.write(ARCH_REGS[random.randrange(0,len(ARCH_REGS))])
            else:
                f.write(ARCH_REGS[random.randrange(0,len(ARCH_REGS))])
                f.write(" ")
                f.write((ARCH_REGS+["X"])[random.randrange(0,len(ARCH_REGS) + 1)])
                f.write(" ")
                f.write((ARCH_REGS+["X"])[random.randrange(0,len(ARCH_REGS) + 1)])
            f.write(" ")
            f.write(str(random.randint(0,10)))
            f.write("\n")
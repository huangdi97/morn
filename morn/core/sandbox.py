"""沙箱系统——seccomp A 级沙箱 + 分级沙箱工厂函数"""

import enum
import logging
import platform

logger = logging.getLogger("morn.sandbox")


class SandboxLevel(str, enum.Enum):
    SAME = "same_process"
    SECCOMP = "seccomp"
    NSJAIL = "nsjail"
    FIRECRACKER = "firecracker"


SECCOMP_ALLOWED_SYSCALLS = frozenset({
    0,   # read
    1,   # write
    257, # openat
    3,   # close
    5,   # fstat
    9,   # mmap
    10,  # mprotect
    11,  # munmap
    60,  # exit_group
    13,  # rt_sigaction
    228, # clock_gettime
    35,  # nanosleep
    318, # getrandom
    12,  # brk
    21,  # access
    89,  # readlink
    262, # newfstatat
    39,  # getpid
    186, #gettid
    96,  # gettimeofday
    102, # getuid
    104, # getgid
    107, # geteuid
    108, # getegid
    # asyncio / 网络 IO 必需
    41,  # socket
    42,  # connect
    44,  # sendto
    45,  # recvfrom
    19,  # readv
    20,  # writev
    16,  # ioctl
    4,   # stat
    8,   # lseek
    32,  # dup
    72,  # fcntl
    291, # epoll_create1
    233, # epoll_ctl
    232, # epoll_wait
    290, # eventfd2
    283, # timerfd_create
    286, # timerfd_settime
    202, # futex
    # pipe / 进程通信
    22,  # pipe
    293, # pipe2
    103, # syslog
})

SECCOMP_DENIED_SYSCALLS = frozenset({
    165,  # mount
    166,  # umount2
    167,  # umount
    273,  # unshare
    56,   # clone
    57,   # fork
    58,   # vfork
    101,  # ptrace
    280,  # bpf
    272,  # kexec_file_load
    224,  # swapon
    225,  # swapoff
    176,  # delete_module
    175,  # init_module
    177,  # finit_module
    130,  # personality
    61,   # wait4
    62,   # kill
    157,  # prctl
    61,   # wait4
    87,   # link
    88,   # unlink
    82,   # rmdir
    84,   # mkdir
    86,   # uselib
    63,   # uname
    59,   # execve
    520,  # execveat
})


class Sandbox:
    LEVEL_SAME = "same_process"
    LEVEL_SECCOMP = "seccomp"
    LEVEL_NSJAIL = "nsjail"
    LEVEL_FIRECRACKER = "firecracker"

    def __init__(self, level: str, allowed_paths: list[str] | None = None):
        self.level = level
        self.allowed_paths = allowed_paths or []

    async def __aenter__(self):
        if self.level == self.LEVEL_SECCOMP:
            self._apply_seccomp()
        return self

    async def __aexit__(self, *exc):
        pass

    def _apply_seccomp(self):
        if platform.system() != "Linux":
            logger.warning("seccomp not available on %s — sandbox degraded to no-op", platform.system())
            return
        try:
            import ctypes
            import ctypes.util
            PR_SET_SECCOMP = 22
            SECCOMP_SET_MODE_FILTER = 1

            libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)

            sock_fprog = self._build_bpf_program()
            ptr = ctypes.pointer(sock_fprog)
            ret = libc.prctl(PR_SET_SECCOMP, SECCOMP_SET_MODE_FILTER, ptr)
            if ret != 0:
                errno = ctypes.get_errno()
                raise PermissionError(f"prctl(PR_SET_SECCOMP) failed with errno={errno}")
            logger.info("seccomp BPF filter applied")
        except PermissionError:
            raise
        except Exception as e:
            logger.warning("failed to apply seccomp: %s — sandbox degraded", e)

    def _build_bpf_program(self):
        import ctypes
        BPF_LD = 0x00
        BPF_JMP = 0x05
        BPF_RET = 0x06
        BPF_W = 0x00
        BPF_ABS = 0x20
        BPF_JEQ = 0x10
        BPF_JEQ_R = 0x15
        BPF_JGT = 0x20
        BPF_JGE = 0x30
        BPF_K = 0x00

        SECCOMP_RET_KILL = 0x80000000
        SECCOMP_RET_ALLOW = 0x7fff0000

        class sock_filter(ctypes.Structure):
            _fields_ = [("code", ctypes.c_uint16), ("jt", ctypes.c_uint8),
                        ("jf", ctypes.c_uint8), ("k", ctypes.c_uint32)]

        SYS_MAX = 600
        instructions = []

        instructions.append(sock_filter(code=BPF_LD | BPF_W | BPF_ABS, k=0))
        arch_field = 0
        instructions.append(sock_filter(code=BPF_JMP | BPF_JGE | BPF_K, jt=0, jf=1, k=arch_field))
        instructions.append(sock_filter(code=BPF_RET | BPF_K, k=SECCOMP_RET_KILL))
        instructions.append(sock_filter(code=BPF_LD | BPF_W | BPF_ABS, k=4))

        allowed_list = sorted(SECCOMP_ALLOWED_SYSCALLS)
        for i, nr in enumerate(allowed_list):
            if i == 0:
                if nr > 0:
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JGT | BPF_K, jt=2, jf=1, k=nr - 1
                    ))
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JGT | BPF_K, jt=0, jf=2, k=nr
                    ))
                else:
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JEQ | BPF_K, jt=0, jf=1, k=nr
                    ))
            else:
                prev = allowed_list[i - 1] + 1
                if nr > prev:
                    mid = (prev + nr) // 2
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JGT | BPF_K, jt=2, jf=1, k=mid
                    ))
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JGT | BPF_K, jt=0, jf=2, k=nr
                    ))
                else:
                    instructions.append(sock_filter(
                        code=BPF_JMP | BPF_JEQ | BPF_K, jt=0, jf=1, k=nr
                    ))
            instructions.append(sock_filter(
                code=BPF_RET | BPF_K, k=SECCOMP_RET_ALLOW
            ))

        instructions.append(sock_filter(
            code=BPF_RET | BPF_K, k=SECCOMP_RET_KILL
        ))

        class sock_fprog(ctypes.Structure):
            _fields_ = [("len", ctypes.c_uint16), ("filter", ctypes.POINTER(sock_filter))]

        arr = (sock_filter * len(instructions))(*instructions)
        return sock_fprog(len(instructions), arr)


def get_sandbox_for(plugin_level: str, allowed_paths: list[str] | None = None) -> Sandbox | None:
    level_map = {
        "S": Sandbox.LEVEL_SAME,
        "A": Sandbox.LEVEL_SECCOMP,
        "B": Sandbox.LEVEL_NSJAIL,
        "C": Sandbox.LEVEL_FIRECRACKER,
    }
    mapped = level_map.get(plugin_level, Sandbox.LEVEL_SAME)
    if mapped == Sandbox.LEVEL_SAME:
        return None
    if mapped in (Sandbox.LEVEL_NSJAIL, Sandbox.LEVEL_FIRECRACKER):
        logger.info("sandbox level %s is marked as v1.0 — no isolation applied", mapped)
        return None
    return Sandbox(mapped, allowed_paths)
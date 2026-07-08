import { reactive, computed } from "vue"
import { listen } from "@tauri-apps/api/event"
import { invoke } from "@tauri-apps/api/core"

export type TaskStatus = "idle" | "launching" | "running" | "exited" | "error" | "downloading" | "completed" | "crashed"

export interface Task {
  id: string
  type: "launch" | "install"
  title: string
  status: TaskStatus
  progress: number
  label: string
  error?: string
  instanceId?: string
  gameVersion?: string
  javaVersion?: string
  systemVersion?: string
  crashLog?: string
}

interface State {
  tasks: Task[]
}

const state = reactive<State>({
  tasks: [],
})

const listenerCleanups = new Map<string, () => void>()

export function addTask(task: Task) {
  const idx = state.tasks.findIndex(t => t.id === task.id)
  if (idx >= 0) {
    state.tasks[idx] = { ...state.tasks[idx], ...task }
  } else {
    state.tasks.push(task)
  }
}

export function updateTask(id: string, partial: Partial<Task>) {
  const task = state.tasks.find(t => t.id === id)
  if (task) Object.assign(task, partial)
}

export function removeTask(id: string) {
  const idx = state.tasks.findIndex(t => t.id === id)
  if (idx >= 0) {
    state.tasks.splice(idx, 1)
    const cleanup = listenerCleanups.get(id)
    if (cleanup) {
      cleanup()
      listenerCleanups.delete(id)
    }
  }
}

export function getTask(id: string): Task | undefined {
  return state.tasks.find(t => t.id === id)
}

let nextId = 1
export function generateId(): string {
  return "task_" + (nextId++)
}

export function useTaskStore() {
  return {
    tasks: computed(() => state.tasks),
    activeTasks: computed(() =>
      state.tasks.filter(t => t.status !== "completed" && t.status !== "idle")
    ),
    addTask,
    updateTask,
    removeTask,
    getTask,
  }
}

const stepLabels: Record<string, string> = {
  manifest: "正在获取版本清单...",
  version_json: "正在获取版本信息...",
  client_jar: "正在下载客户端...",
  finalize: "正在处理...",
  done: "安装完成",
}

export async function registerLaunchListeners(instanceName: string) {
  const taskId = "launch:" + instanceName
  if (listenerCleanups.has(taskId)) return

  const unlisteners: (() => void)[] = []

  const u1 = await listen<{ kind: string; downloaded: number; total: number }>("minecraft-progress", (event) => {
    const { kind, downloaded, total } = event.payload
    const pct = total > 0 ? downloaded / total : 0
    updateTask(taskId, {
      status: "launching",
      progress: pct,
      label: kind ? `正在下载 ${kind}...` : "启动中...",
    })
  })
  unlisteners.push(u1)

  const u2 = await listen<{ line: string }>("minecraft-output", () => {})
  unlisteners.push(u2)

  const u3 = await listen("minecraft-ready", () => {
    updateTask(taskId, { status: "running", label: "停止", progress: 1 })
  })
  unlisteners.push(u3)

  const u4 = await listen<{ message: string }>("minecraft-error", (event) => {
    updateTask(taskId, { status: "error", label: "启动失败: " + event.payload.message })
    const t = getTask(taskId)
    if (t) {
      invoke("open_crash_shell", {
        report: {
          instanceName: t.title,
          gameVersion: t.gameVersion || "未知",
          javaVersion: t.javaVersion || "未知",
          systemVersion: t.systemVersion || navigator.userAgent,
          errorMessage: event.payload.message,
          crashLog: event.payload.message,
          solution: getSolution(event.payload.message),
        },
      }).catch(() => {})
    }
    const cleanup = listenerCleanups.get(taskId)
    if (cleanup) { cleanup(); listenerCleanups.delete(taskId) }
  })
  unlisteners.push(u4)

  const u5 = await listen<{ code: number }>("minecraft-exit", (event) => {
    if (event.payload.code !== 0) {
      updateTask(taskId, { status: "crashed", label: "游戏异常退出 (代码 " + event.payload.code + ")", progress: 1 })
      const t = getTask(taskId)
      if (t) {
        invoke("open_crash_shell", {
          report: {
            instanceName: t.title,
            gameVersion: t.gameVersion || "未知",
            javaVersion: t.javaVersion || "未知",
            systemVersion: t.systemVersion || navigator.userAgent,
            errorMessage: "游戏异常退出，退出代码: " + event.payload.code,
            crashLog: "游戏进程以非零退出码终止。退出代码: " + event.payload.code,
            solution: getCrashExitSolution(event.payload.code),
          },
        }).catch(() => {})
      }
    } else {
      updateTask(taskId, { status: "exited", label: "游戏已退出", progress: 1 })
      setTimeout(() => {
        const t = getTask(taskId)
        if (t && t.status === "exited") {
          removeTask(taskId)
        }
      }, 3000)
    }
    const cleanup = listenerCleanups.get(taskId)
    if (cleanup) { cleanup(); listenerCleanups.delete(taskId) }
  })
  unlisteners.push(u5)

  listenerCleanups.set(taskId, () => {
    unlisteners.forEach(fn => fn())
  })
}

function getSolution(msg: string): string {
  if (msg.includes("OutOfMemoryError") || msg.includes("out of memory") || msg.includes("Not enough memory")) {
    return "内存不足。请尝试在实例设置中增加分配给 Minecraft 的内存（最大内存）。"
  }
  if (msg.includes("UnsatisfiedDependencyError") || msg.includes("NoClassDefFoundError") || msg.includes("ClassNotFoundException")) {
    return "缺少依赖或模组文件损坏。请尝试重新安装当前游戏版本或检查模组兼容性。"
  }
  if (msg.includes("OpenGL") || msg.includes("GLX") || msg.includes("Pixel format")) {
    return "显卡驱动问题。请更新您的显卡驱动程序，或尝试在设置中切换渲染器。"
  }
  if (msg.includes("Java")) {
    return "Java 运行时错误。请检查您的 Java 安装是否完整，或尝试安装其他 Java 版本。"
  }
  return "未知错误。请查看下方崩溃日志获取详细信息，或将日志提供给开发者以获取帮助。"
}

function getCrashExitSolution(code: number): string {
  if (code === -1) return "进程被信号终止。可能是系统资源不足或进程被强制关闭。"
  if (code === 1) return "游戏启动器返回错误代码 1。请检查游戏版本和模组配置是否正确。"
  return `游戏异常退出 (代码 ${code})。请查看下方崩溃日志获取详细信息。`
}

export async function registerInstallListeners(taskId: string) {
  if (listenerCleanups.has(taskId)) return

  const unlisteners: (() => void)[] = []

  const u1 = await listen<{ step: string; progress: number }>("install-progress", (event) => {
    const { step, progress } = event.payload
    updateTask(taskId, {
      status: step === "done" ? "completed" : "downloading",
      progress,
      label: stepLabels[step] || step,
    })
    if (step === "done") {
      setTimeout(() => {
        removeTask(taskId)
      }, 3000)
      const cleanup = listenerCleanups.get(taskId)
      if (cleanup) { cleanup(); listenerCleanups.delete(taskId) }
    }
  })
  unlisteners.push(u1)

  listenerCleanups.set(taskId, () => {
    unlisteners.forEach(fn => fn())
  })
}

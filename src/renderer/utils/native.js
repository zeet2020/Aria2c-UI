import { shell, nativeTheme } from '@electron/remote'
import { ElMessage as Message } from 'element-plus'

import {
  getFileNameFromFile,
  isMagnetTask
} from '@shared/utils'
import { APP_THEME, TASK_STATUS } from '@shared/constants'

// Browser-safe path join. aria2 returns absolute paths; we only need to
// normalize and join a name onto a directory. Pick the separator from the
// existing path so Windows ("\\") and POSIX ("/") both work.
const joinPath = (...parts) => {
  const segs = parts.filter(Boolean)
  if (segs.length === 0) return ''
  const sep = /\\/.test(segs[0]) ? '\\' : '/'
  return segs
    .map((s, i) => (i === 0 ? s.replace(/[\\/]+$/, '') : s.replace(/^[\\/]+|[\\/]+$/g, '')))
    .join(sep)
}

export const showItemInFolder = async (fullPath, { errorMsg } = {}) => {
  if (!fullPath) {
    return
  }
  try {
    await shell.showItemInFolder(fullPath)
  } catch (err) {
    console.warn(`[AUI] showItemInFolder failed: ${fullPath}`, err)
    if (errorMsg) {
      Message.error(errorMsg)
    }
  }
}

export const openItem = async (fullPath) => {
  if (!fullPath) {
    return
  }
  const result = await shell.openPath(fullPath)
  return result
}

export const getTaskFullPath = (task) => {
  const { dir, files, bittorrent } = task
  let result = dir

  // Magnet link task
  if (isMagnetTask(task)) {
    return result
  }

  if (bittorrent && bittorrent.info && bittorrent.info.name) {
    result = joinPath(result, bittorrent.info.name)
    return result
  }

  const [file] = files
  const path = file.path ? file.path : ''
  let fileName = ''

  if (path) {
    result = path
  } else {
    if (files && files.length === 1) {
      fileName = getFileNameFromFile(file)
      if (fileName) {
        result = joinPath(result, fileName)
      }
    }
  }

  return result
}

export const moveTaskFilesToTrash = async (task) => {
  /**
   * For magnet link tasks, there is bittorrent, but there is no bittorrent.info.
   * The path is not a complete path before it becomes a BT task.
   * In order to avoid accidentally deleting the directory
   * where the task is located, it directly returns true when deleting.
   */
  if (isMagnetTask(task)) {
    return true
  }

  const { dir, status } = task
  const path = getTaskFullPath(task)
  if (!path || dir === path) {
    throw new Error('task.file-path-error')
  }

  // shell.trashItem is backed by a Rust command that no-ops on missing files.
  const deleteResult1 = await shell.trashItem(path)

  // There is no configuration file for the completed task.
  if (status === TASK_STATUS.COMPLETE) {
    return deleteResult1
  }

  const deleteResult2 = await shell.trashItem(`${path}.aria2`)

  return deleteResult1 && deleteResult2
}

export const getSystemTheme = () => {
  return nativeTheme.shouldUseDarkColors ? APP_THEME.DARK : APP_THEME.LIGHT
}

export const delayDeleteTaskFiles = (task, delay) => {
  return new Promise((resolve, reject) => {
    setTimeout(async () => {
      try {
        const result = await moveTaskFilesToTrash(task)
        resolve(result)
      } catch (err) {
        reject(err.message)
      }
    }, delay)
  })
}

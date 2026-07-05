#include "vidcut/CommandManager.h"

namespace VidCut {

void CommandManager::execute(Command* cmd) {
    cmd->execute();
    m_undoStack.push(cmd);
    // New action clears redo history
    qDeleteAll(m_redoStack);
    m_redoStack.clear();
}

void CommandManager::undo() {
    if (m_undoStack.isEmpty()) return;
    Command* cmd = m_undoStack.pop();
    cmd->undo();
    m_redoStack.push(cmd);
}

void CommandManager::redo() {
    if (m_redoStack.isEmpty()) return;
    Command* cmd = m_redoStack.pop();
    cmd->execute();
    m_undoStack.push(cmd);
}

void CommandManager::clear() {
    qDeleteAll(m_undoStack);
    m_undoStack.clear();
    qDeleteAll(m_redoStack);
    m_redoStack.clear();
}

} // namespace VidCut

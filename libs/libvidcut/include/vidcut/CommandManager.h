#pragma once
#include <functional>
#include <QStack>

namespace VidCut {

// Command interface for undo/redo.
struct Command {
    virtual ~Command() = default;
    virtual void execute() = 0;
    virtual void undo() = 0;
};

class CommandManager {
public:
    void execute(Command* cmd);
    void undo();
    void redo();

    bool canUndo() const { return !m_undoStack.isEmpty(); }
    bool canRedo() const { return !m_redoStack.isEmpty(); }

    void clear();

private:
    QStack<Command*> m_undoStack;
    QStack<Command*> m_redoStack;
};

} // namespace VidCut

package com.modrinth.theseus.agent.transformers;

import java.util.ListIterator;
import org.objectweb.asm.Opcodes;
import org.objectweb.asm.tree.AbstractInsnNode;
import org.objectweb.asm.tree.ClassNode;
import org.objectweb.asm.tree.InsnNode;
import org.objectweb.asm.tree.MethodInsnNode;
import org.objectweb.asm.tree.MethodNode;

public final class IntegratedServerTransformer extends ClassNodeTransformer {
    @Override
    protected boolean transform(ClassNode classNode) {
        boolean modified = false;
        for (final MethodNode method : classNode.methods) {
            final ListIterator<AbstractInsnNode> it = method.instructions.iterator();
            while (it.hasNext()) {
                final AbstractInsnNode insn = it.next();
                if (insn instanceof MethodInsnNode) {
                    final MethodInsnNode methodInsn = (MethodInsnNode) insn;
                    if (methodInsn.desc.equals("(Z)V")
                            && (methodInsn.name.equals("setOnlineMode")
                                    || methodInsn.name.equals("func_71229_d")
                                    || methodInsn.name.equals("method_3812")
                                    || methodInsn.name.contains("setOnlineMode"))) {
                        method.instructions.insertBefore(methodInsn, new InsnNode(Opcodes.POP));
                        method.instructions.insertBefore(methodInsn, new InsnNode(Opcodes.ICONST_0));
                        modified = true;
                    }
                }
            }
        }
        return modified;
    }
}

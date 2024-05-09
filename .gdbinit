
source /root/.gdbinit-gef.py

file esp/KERNEL.ELF
gef-remote localhost 1234
tmux-setup
// b ysos_kernel::init
// b new_stack_test_thread
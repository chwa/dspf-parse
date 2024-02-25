Faster parsing / progress status
================================

- parse header separately (to first *|NET section)
- split by *|NET section (until .ENDS)
  - can be parsed by separate threads
  - completion of section can update progress percentage

"Capacitance for net" feature
=============================

let net: Net = ...;

for subnode_idx in net.sub_nodes:


User interface
==============

1) File selection (skip if passed on command line)

+---------------------------------+
| /my/current/path/               |    <-.
+---------------------------------+      |  tab to toggle
|    ../                          |    <-'
|    folder1/                     |
|    unrelated.txt            34k |
| >> circuit_trcp70.dspf     156M |    <--- enter to select
+---------------------------------+

2) Progress bar

+---------------------------------+
|                                 |
|   Loading circuit_trcp70.dspf   |
|   ▏████████                 ▕   |
|                                 |
|         Ctrl-C to abort         |
+---------------------------------+

3) Analysis selection

+---------------------------------+
| File: circuit_trcp70.dspf       |
| 123 Nets, 7347 Subnodes, ...    |
+---------------------------------+
| >> Capacitance for net...       |
|    Capacitance between nets...  |
|    Path resistance...           |
+---------------------------------+

4) Net/options form

+---------------------------------+
|    XXabc/XImodule<0>/net3       |
| >> XXabc/XImodule<0>/net35      |
|    XXabc/XImodule<0>/net36      |
|    XXabc/XImodule<1>/abc        |
+---------------------------------+
| *net35                          |   <- wildcard search?
+---------------------------------+

5) Result (by net, or by layer)

+-------------------+-------------+
| XXXabc/net123     | ███████████ |
| XXmyinst/clk      | ███████     |
| XXmyinst/clk      | ███     7.5 |
| XXmyinst/reset    | ██      3.3 |
+-------------------+-------------+
| 345 nets          | Total: 73.4 |
+-------------------+-------------+

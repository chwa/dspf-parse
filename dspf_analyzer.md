# `dspf_analyzer`

## Invocation

```
dspf_analyzer /path/to/file.dspf
```

Definitions:
- Nets: Net sections in the DSPF file (`*|NET`), equivalent to schematic nets.
- Subnodes: Nodes of the segmented net (when extracting R). Denoted by `*|S` in the DSF file.

## Report capacitance for net...

Use the arrow keys to navigate between the 3 panes, and to select a specific net to display.

The first 2 panes have a wildcard filter. Use a wildcard expression (using `*` and `?`) to quickly
find nets. Use `*` to show all nets.

The 'Aggressor net:' pane shows the total value of all parasitic caps that are directly connected
to the selected victim, and the contribution (value and percentage) from specific nets.

The 'Layer pairs:' pane breaks down the selected value from the middle pane (either the total, or a
specific net) by the layer annotations given in the DSPF file. 'Self' refers to the victim net,
'other' to the aggressor.

Press `<space>` to toggle between different views for the layer information:
- Individual layer pairs
- Grouped by 'self' layer
- Grouped by 'other' layer.

(This window currently does not support scrolling, make your terminal window larger to see more
values...)


## Path resistance [experimental]...

*As the name indicates, this has not been extensively tested, but it seems to work...*

For this analysis, you define one or more subnodes to be the 'inputs'. These nodes will be shorted
together, and tied to a fixed voltage (0V). You then define one or more subnodes to be the
'outputs'. These nodes are assumed to all draw an equal current. For n outputs, each output node
draws 1/n A.


1. On the first page, select the net (you can use a wildcard expression to quickly find a name).
1. The subnodes of that net will now be displayed under input nodes/output nodes. The 'input nodes'
pane shows all subnodes that match the current wildcard expression (at the bottom). Here you
would typically enter a single subnode, for example the main node (=pin name) of a net, like `out`.
1. The 'output nodes' pane shows all the remaining nodes that match the wildcard expression for
outputs. Here you can write a wildcard expression like `XI24/MM2<*>#d` to select the terminals of a
specific device (the naming of nodes depends on the settings that were used for extraction).

**In the input/output panes, you *must* use the wildcard entries to select a set of nodes.** You
are done when the 2 panes show the nodes that you want (cursor selection does nothing).

The results table then displays the 'equivalent resistance' values for the output nodes. This value
is the IR drop in volts resulting from all nodes being loaded by 1/n A. **Note that in general,
this value is not equal to the point-to-point resistance between a single input and a single output
node.** If the netlist consists of 10 parallel resistors of 10 Ohm going from the input node to 10
output nodes, the reported 'equivalent resistances' would be **1** Ohm for all nodes, and 1 Ohm
'total effective R'.

The 'Total effective R' reported in the top right is the value of the single resistor that would
dissipate the same power at 1A as the real network of resistors. This is equal to the mean of the
individual R values reported below it.

The table at the bottom shows a breakdown of the total resistance by layer.


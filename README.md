# pafnet

PAF (pairwise alignment format) to [network/graph](https://gephi.org/users/supported-graph-formats/) format converter.
This tool is designed to project PAF alignments (the implied overlap and containment graph) into a network format that can be processed by tools like [grappolo](https://github.com/Exa-Graph/grappolo) and [gephi](https://gephi.org/)

## usage

For instance, to convert into Pajek NET format:

```
pafnet -n x.paf >x.net
```

Or to write an edgelist:

```
pafnet -e x.paf >x.edges
```

Or, to apply colors to the nodes based on a specifed group name that the sequences in the PAF file are prefixed with, producing GEXF which can be rendered in [gephi](http://gephi.org/):

```
# cat colors.rgb
group1  255     0       0
group2  255     96      0
group3  255     191     0
group4  223     255     0
group5  128     255     0
group6  32      255     0
group7  0       255     64
group8  0       255     159
group9  0       255     255
group10 0       159     255
group11 0       64      255
group12 32      0       255
group13 128     0       255
group14 223     0       255
group15 255     0       191
group16 255     0       96
```

making GEXF:

```
pafnet -c colors.rgb -p . -g aln.paf >graph.gexf
```

The sequence names should look like `group3.XXXXXX` for this to work.

## building

Build using cargo. This is rust :)

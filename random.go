package main

import (
	"math/rand"
	"time"
)

type Node struct {
	name     string
	children []Node
}

func newNode(name string) *Node {
	children := make([]Node, 0)
	return &Node{
		name,
		children,
	}
}

func (n *Node) add(nodes ...*Node) *Node {
	for _, node := range nodes {
		n.children = append(n.children, *node)
	}
	return n
}

func (n *Node) exec(f func(node *Node)) {
	f(n)
}

func randSelect(node *Node) {
	print(node.name)
	if len(node.children) <= 0 {
		print("\n")
		return
	}
	print(" -> ")
	index := rand.Int31n(int32(len(node.children)))
	node.children[index].exec(randSelect)
}

func newRoot() *Node {
	root := newNode("where to eat:")
	nList := []*Node{
		newNode("一楼"),
		newNode("二楼"),
		newNode("三楼"),
	}
	root.add(
		newNode("东苑").add(
			newNode("东一").add(nList...),
			newNode("东二").add(nList...),
			newNode("芳缘"),
		),
		newNode("西苑").add(nList...),
		newNode("软件园").add(nList...),
	)
	return root
}

func main() {
	rand.Seed(time.Now().Unix())
	for i := 0; i < int(rand.Int31n(1000) + 100); i++ {
		newRoot().exec(randSelect)
	}
}

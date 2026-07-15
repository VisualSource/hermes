export const createBlockNode = (createNode) => {
	return (parentNode, children, match, isImport) => {
		const node = createNode(match);
		node.append(...children);
		parentNode.replace(node);
		if (!isImport) {
			node.select(0, 0);
		}
	};
};

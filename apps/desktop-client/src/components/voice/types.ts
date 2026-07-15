export type UserCard = {
	type: "user";
	id: string;
	avatar: string;
	username: string;
	color: string;
};

export type StreamCard = {
	type: "stream";
	preview: string;
	id: string;
	fps: number;
	res: number;
};

export type CardType = UserCard | StreamCard;

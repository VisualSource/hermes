import type { CardType, StreamCard } from "@/components/voice/types";
import { faker } from "@faker-js/faker";
import { useState } from "react";

const items: CardType[] = [
	{
		type: "stream",
		preview: faker.image.urlPicsumPhotos({ blur: 2 }),
		id: faker.string.ulid(),
		res: 720,
		fps: 30,
	},
	{
		type: "stream",
		preview: faker.image.urlPicsumPhotos({ blur: 2 }),
		id: faker.string.ulid(),
		fps: 60,
		res: 1080,
	},
	...(Array.from({ length: 11 }).map(() => ({
		type: "user",
		id: faker.string.uuid(),
		avatar: faker.image.avatarGitHub(),
		username: faker.person.firstName(),
		color: faker.color.human(),
	})) as CardType[]),
];

export const useVoice = () => {
	const [watching, setWatching] = useState<string | null>(null);

	return {
		watch: (id: string | null) => setWatching(id),
		items,
		watchingStream: items.find((e) => e.id === watching) as
			| StreamCard
			| undefined,
	};
};

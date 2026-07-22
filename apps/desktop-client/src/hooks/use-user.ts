import { UserStatus, type User } from "@/lib/api/types";
import { faker } from "@faker-js/faker";
import { UUID } from "./use-app";


const fakeData_user: User = {
	id: faker.string.uuid() as UUID,
	username: faker.person.firstName(),
	avatar: faker.image.avatarGitHub(),
	status: UserStatus.Online,
	roles: BigInt(0),
};

export const useUser = () => {
	return fakeData_user;
};
 
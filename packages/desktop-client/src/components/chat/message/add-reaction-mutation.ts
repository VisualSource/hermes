import { useUser } from "@/hooks/use-user";
import { useMutation } from "@tanstack/react-query";

export const useAddReactionMutation = () => {
	const user = useUser();
	const { mutateAsync } = useMutation<unknown, Error, string>({
		async mutationFn(id) {},
		onError() {},
	});

	return {
		mutateAsync,
	};
};

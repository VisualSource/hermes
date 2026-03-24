export const UserAvatar = ({ userId: string }:{ userId: string; }) => {
  const { data, isLoading, error, isError } = useQuery({
    select: (data) => data.find(user=>user.id === userId),
    queryKey: ["user-server","SERVER_ID"],
    queryFn: async () => {
      //TODO: query server users profile for server
      return [];
    }
  });

  return (
    <Avatar>
      <AvatarImage href={data.avatar}/>
      <AvatarFallback>UN</AvatarFallback>
    </Avatar>
  );
}

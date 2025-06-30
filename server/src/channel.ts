class Channel {
    users = new Map<string, unknown>();

    constructor(private uuid: string, private name: string) { }

    join(userId: string, details: unknown) {
        this.users.set(userId, details);

        return {}
    }

    leave(userId: string) {
        this.users.delete(userId);
    }

    toJson() {
        return {
            channelId: this.uuid,
            name: this.name
        }
    }
}

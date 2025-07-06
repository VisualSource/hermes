export class RoomManager {
    static INSTANCE: RoomManager | null = null;
    static get() {
        if (!RoomManager.INSTANCE) {
            RoomManager.INSTANCE = new RoomManager();
        }

        return RoomManager.INSTANCE;
    }

    private rooms: Map<string, Set<string>> = new Map();



    addRoom(roomId: string) {
        this.rooms.set(roomId, new Set());
    }
    removeRoom(roomId: string) {
        this.rooms.delete(roomId);
    }
    hasRoom(roomId: string) {
        return this.rooms.has(roomId);
    }

    addUserToRoom(roomId: string, userId: string) {
        const room = this.rooms.get(roomId);
        if (room) {
            room.add(userId);
        }
    }
    removeUserFromRoom(roomId: string, userId: string) {
        const room = this.rooms.get(roomId);
        if (room) {
            room.delete(userId);
        }
    }
    roomHasUser(roomId: string, userId: string) {
        return this.rooms.get(roomId)?.has(userId) ?? false;
    }

    getRoomUsers(roomId: string) {
        const room = this.rooms.get(roomId);
        if (!room) return null;

        return Array.from(room.values());
    }
}
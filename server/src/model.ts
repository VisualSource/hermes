export class User {
    id = ""
    username = ""
    psdHash = ""
    icon = ""

    public toJson() {
        return {
            id: this.id,
            username: this.username,
            icon: this.icon
        }
    }
}

export class Channel {
    id = ""
    members = "";
    name = ""
    icon = ""
    admins = "";

    get channelMembers() {
        return JSON.parse(this.members) as string[];
    }
    get channelAdmins() {
        return JSON.parse(this.admins) as string[];
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
            icon: this.icon,
            members: this.channelMembers,
            admins: this.channelAdmins
        }
    }
}

export class Room {
    id = "";
    type: "text" | "media" = "text"
    name = "";
}
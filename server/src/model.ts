export class User {
    id = ""
    username = ""
    psdHash = ""
    icon = ""

    public toJson() {
        return {
            id: this.id,
            username: this.username
        }
    }
}
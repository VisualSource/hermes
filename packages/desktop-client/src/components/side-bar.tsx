import { Avatar } from "./ui/avatar"
import { Select } from "./ui/select"
import { Separator } from "./ui/separator"

export const SideBar = () => {
    return (
        <aside>
            <Select>

            </Select>
            <ul>
                <li>$ Roles Channel</li>
                <li># text channel</li>
                <li>% voice channel</li>
                <li>
                    <Separator/>
                </li>
                <li>
                    <ul>
                        <li>
                            # Grouped channel
                        </li>
                    </ul>
                </li>
            </ul>
            <div>
                <div>
                    <Avatar>

                    </Avatar>
                    <h1>Username</h1>
                    <p>Status</p>
                </div>
                <div>
                    <h1>RTC Connecting</h1>
                </div>
                <div>
                    <button>Mute</button>
                    <button>End</button>
                </div>
            </div>
        </aside>
    )
}
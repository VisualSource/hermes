import { use } from "react";
import { AppContext } from "@/lib/appContext";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";

export const UserIcon: React.FC<{ userId: string }> = ({ userId }) => {
    const ctx = use(AppContext);
    if (!ctx) throw new Error("No auth context");
    const profile = use(ctx.auth.hookGetUser(userId));

    return (
        <AvatarImage src={profile.icon} />
    );
}

export const UserAvatar: React.FC<{ userId: string }> = ({ userId }) => {
    const ctx = use(AppContext);
    if (!ctx) throw new Error("No auth context");
    const profile = use(ctx.auth.hookGetUser(userId));

    return (
        <Avatar>
            <AvatarImage src={profile.icon} />
            <AvatarFallback>

            </AvatarFallback>
        </Avatar>
    );
}

export const NamePlate: React.FC<{ userId: string }> = ({ userId }) => {
    const ctx = use(AppContext);
    if (!ctx) throw new Error("No auth context");
    const profile = use(ctx.auth.hookGetUser(userId));

    return (
        <li className="inline-flex gap-2 items-center">
            <Avatar>
                <AvatarImage src={profile.icon} />
                <AvatarFallback>

                </AvatarFallback>
            </Avatar>
            <h4>
                {profile.username}
            </h4>
        </li>
    );
}
import { Avatar } from "@/components/ui/avatar";
import { Select, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/settings")({
	component: Index,
});

function Index() {
	return (
		<main>
			<Tabs orientation="vertical" defaultValue="app">
				<TabsList>
					<TabsTrigger value="app">App</TabsTrigger>
					<TabsTrigger value="user">User</TabsTrigger>
					<TabsTrigger value="overlay">Overlay</TabsTrigger>
					<TabsTrigger value="voice">Voice</TabsTrigger>
				</TabsList>
				<TabsContent value="app">
					<div>
						<div>
							<label>Theme</label>
							<div>
								<Select>
								<SelectTrigger>
									<SelectValue/>
								</SelectTrigger>
							</Select>
							<Button>Themes Dir</Button>
							</div>
						</div>
						<div>
							<label>Dark Mode</label>
							<Switch/>
						</div>
						<div>
							App Version: 0.1.0 
							Tauri Version: 1.7.3
							channel: stable,
							overlay: 0.1.0
						</div>
					</div>
				</TabsContent>
				<TabsContent value="user">
					<div>
						<Avatar>

						</Avatar>
						<p>Username</p>
					</div>
					<Button>Edit</Button>
					<Button>Logout</Button>
				</TabsContent>
				<TabsContent value="overlay">
					<div>
						<label>Enable Overlay</label>
						<Select>
							<SelectTrigger>
								<SelectValue/>
							</SelectTrigger>
						</Select>
					</div>

					<div>
						<label>Injected Games</label>
						<ul>
							<li>
								<div>
									<h1>Game Title</h1>
									<p>Process name</p>
								</div>
								<button>X</button>
							</li>
						</ul>
					</div>
						
				</TabsContent>
				<TabsContent value="voice">
					<label>Input</label>
					<Select>
						<SelectTrigger>
							<SelectValue/>
						</SelectTrigger>
					</Select>
					<div>
						<label>Video</label>
						<Select>
							<SelectTrigger>
								<SelectValue/>
							</SelectTrigger>
						</Select>
					</div>
				</TabsContent>
			</Tabs>
		</main>
	);
}
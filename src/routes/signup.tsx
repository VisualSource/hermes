import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/hooks/use-auth';
import { toast } from "sonner"
import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { GalleryVerticalEnd } from 'lucide-react';

import { useFormStatus } from 'react-dom';
import { useState } from 'react';

export const Route = createFileRoute('/signup')({
  component: RouteComponent,
})

const FormSubmitButton: React.FC = () => {
  const { pending } = useFormStatus();
  return (
    <Button disabled={pending} type="submit" className="w-full">
      Signup
    </Button>
  );
}

function RouteComponent() {
  const auth = useAuth();
  const navigate = useNavigate();
  const [psd, setPsd] = useState<string>("");
  const [checkPsd, setCheckPsd] = useState<string>("");

  return (
    <div className="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
      <div className="flex w-full max-w-sm flex-col gap-6">
        <a href="#" className="flex items-center gap-2 self-center font-medium">
          <div className="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
            <GalleryVerticalEnd className="size-4" />
          </div>
          LoadingZone Hermes
        </a>
        <div className="flex flex-col gap-6">
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="text-xl">Welcome</CardTitle>
              <CardDescription>
                Signup to get started
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form action={async (ev) => {
                try {
                  const username = ev.get("username")?.toString();
                  const password = ev.get("password")?.toString();

                  if (!username || !password) throw new Error("Missing username or password");

                  await auth.signup(username, password);

                  await navigate({ to: "/" })
                } catch (error) {
                  if (Error.isError(error)) {
                    toast("Failed to login", { description: error.message })
                  }
                  console.error(error);
                }
              }}>
                <div className="grid gap-6">
                  <div className="grid gap-6">
                    <div className="grid gap-3">
                      <Label htmlFor="username">Username</Label>
                      <Input
                        id="username"
                        name="username"
                        type="text"
                        placeholder="hermes"
                        minLength={3} maxLength={200}
                        required
                      />
                    </div>
                    <div className="grid gap-3">
                      <Label htmlFor="password">Password</Label>
                      <Input value={psd} onChange={(ev) => setPsd(ev.target.value)} name="password" minLength={8} maxLength={200} id="password" type="password" required />
                    </div>
                    <div className="grid gap-3">
                      <Label htmlFor="password-check">Reenter Password</Label>
                      <Input
                        onChange={(ev) => {

                          if (ev.target.value !== psd) {
                            ev.target.setCustomValidity("Does not match password");
                          } else {
                            ev.target.setCustomValidity("");
                          }

                          setCheckPsd(ev.target.value)
                        }}
                        value={checkPsd} name="password-check"
                        minLength={8}
                        maxLength={200}
                        id="password-check"
                        type="password"
                        required />
                    </div>
                    <FormSubmitButton />
                  </div>
                  <div className="text-center text-sm">
                    Have an account?{" "}
                    <Link to="/signup" className="underline underline-offset-4">
                      Login
                    </Link>
                  </div>
                </div>
              </form>
            </CardContent>
          </Card>
          <div className="text-muted-foreground *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4">
            By clicking continue, you agree to our <a href="#">Terms of Service</a>{" "}
            and <a href="#">Privacy Policy</a>.
          </div>
        </div>
      </div>
    </div>
  );
}
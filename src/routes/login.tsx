import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/hooks/use-auth';
import { toast } from "sonner"
import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { GalleryVerticalEnd } from 'lucide-react';

import { useFormStatus } from 'react-dom';

export const Route = createFileRoute('/login')({
  component: RouteComponent,
})

const FormSubmitButton: React.FC = () => {
  const { pending } = useFormStatus();
  return (
    <Button disabled={pending} type="submit" className="w-full">
      Login
    </Button>
  );
}

function RouteComponent() {
  const auth = useAuth();
  const navigate = useNavigate();

  return (
    <div className="bg-muted flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
      <div className="flex w-full max-w-sm flex-col gap-6">
        <a href="#loadingZone" className="flex items-center gap-2 self-center font-medium">
          <div className="bg-primary text-primary-foreground flex size-6 items-center justify-center rounded-md">
            <GalleryVerticalEnd className="size-4" />
          </div>
          LoadingZone Hermes
        </a>
        <div className="flex flex-col gap-6">
          <Card>
            <CardHeader className="text-center">
              <CardTitle className="text-xl">Welcome back</CardTitle>
              <CardDescription>
                Login with your account
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form action={async (ev) => {
                try {
                  const username = ev.get("username")?.toString();
                  const password = ev.get("password")?.toString();

                  if (!username || !password) throw new Error("Missing username or password");

                  await auth.login(username, password);

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
                      <div className="flex items-center">
                        <Label htmlFor="password">Password</Label>
                        <a
                          href="#loadingZonePassword"
                          className="ml-auto text-sm underline-offset-4 hover:underline"
                        >
                          Forgot your password?
                        </a>
                      </div>
                      <Input name="password" minLength={8} maxLength={200} id="password" type="password" required />
                    </div>
                    <FormSubmitButton />
                  </div>
                  <div className="text-center text-sm">
                    Don&apos;t have an account?{" "}
                    <Link to="/signup" className="underline underline-offset-4">
                      Sign up
                    </Link>
                  </div>
                </div>
              </form>
            </CardContent>
          </Card>
          <div className="text-muted-foreground *:[a]:hover:text-primary text-center text-xs text-balance *:[a]:underline *:[a]:underline-offset-4">
            By clicking continue, you agree to our <a href="#LoadingZoneTermsOfService">Terms of Service</a>{" "}
            and <a href="#loadingZonePrivacyPolicy">Privacy Policy</a>.
          </div>
        </div>
      </div>
    </div>
  );
}
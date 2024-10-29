import { useCallback, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Github, Mail } from "lucide-react";
import axios from "axios";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import "../src/app/globals.css";
import { useRouter } from "next/navigation";

export default function AuthPage() {
  const [isSignUp, setIsSignUp] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();

  const signIn = useCallback(
    (email: string, password: string) => {
      (async () => {
        try {
          const results = await axios.post("http://localhost:8080/sessions", {
            email,
            password,
          });

          document.cookie = `Authorization=${encodeURIComponent(`Bearer ${results.data.token}`)};`;

          router.push("/dashboard");
        } catch (e) {
          console.log("Error:", e);
        }
      })();
    },
    [router],
  );

  const createUser = useCallback((evt: any) => {
    evt.preventDefault();
    evt.stopPropagation();
    (async () => {
      const name = evt.target.name.value;
      const email = evt.target.email.value;
      const password = evt.target.password.value;

      try {
        const results = await axios.post("http://localhost:8080/users", {
          name,
          email,
          password,
        });

        signIn(email, password);
      } catch (e) {
        console.log("Error:", e);
      }
    })();
  }, []);

  const createSession = useCallback((evt: any) => {
    evt.preventDefault();
    evt.stopPropagation();
    (async () => {
      const email = evt.target.email.value;
      const password = evt.target.password.value;

      signIn(email, password);
    })();
  }, []);

  const toggleMode = () => setIsSignUp(!isSignUp);

  return (
    <div className="min-h-screen bg-gradient-to-b from-green-400 to-green-600 flex items-center justify-center p-4">
      <Card className="w-full max-w-md overflow-hidden">
        <CardHeader className="space-y-1">
          <div className="flex justify-between items-center">
            <CardTitle className="text-2xl font-bold">
              {isSignUp ? "Create an account" : "Sign in to your account"}
            </CardTitle>
            <Button
              variant="ghost"
              className="text-green-600 hover:text-green-700 hover:bg-green-100"
              onClick={toggleMode}
            >
              {isSignUp ? "Sign in" : "Sign up"}
            </Button>
          </div>
          <CardDescription>
            {isSignUp
              ? "Enter your email below to create your account"
              : "Enter your credentials to access your account"}
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4">
          <AnimatePresence mode="wait">
            <motion.div
              key={isSignUp ? "signup" : "signin"}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              transition={{ duration: 0.2 }}
            >
              <form
                className="grid gap-4"
                onSubmit={isSignUp ? createUser : createSession}
              >
                <div className="grid gap-2">
                  {isSignUp && (
                    <div className="grid gap-1">
                      <Label htmlFor="name">Name</Label>
                      <Input id="name" type="text" placeholder="John Doe" />
                    </div>
                  )}
                  <div className="grid gap-1">
                    <Label htmlFor="email">Email</Label>
                    <Input
                      id="email"
                      type="email"
                      placeholder="m@example.com"
                    />
                  </div>
                  <div className="grid gap-1">
                    <Label htmlFor="password">Password</Label>
                    <Input id="password" type="password" />
                  </div>
                  {isSignUp && (
                    <div className="grid gap-1">
                      <Label htmlFor="confirm-password">Confirm Password</Label>
                      <Input id="confirm-password" type="password" />
                    </div>
                  )}
                </div>
                <Button className="w-full bg-green-600 hover:bg-green-700">
                  {isSignUp ? "Create account" : "Sign in"}
                </Button>
              </form>
            </motion.div>
          </AnimatePresence>
          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">
                Or continue with
              </span>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-6">
            <Button variant="outline">
              <Github className="mr-2 h-4 w-4" />
              Github
            </Button>
            <Button variant="outline">
              <Mail className="mr-2 h-4 w-4" />
              Google
            </Button>
          </div>
        </CardContent>
        <CardFooter>
          <p className="px-8 text-center text-sm text-muted-foreground">
            By clicking continue, you agree to our{" "}
            <a
              href="#"
              className="underline underline-offset-4 hover:text-primary"
            >
              Terms of Service
            </a>{" "}
            and{" "}
            <a
              href="#"
              className="underline underline-offset-4 hover:text-primary"
            >
              Privacy Policy
            </a>
            .
          </p>
        </CardFooter>
      </Card>
    </div>
  );
}

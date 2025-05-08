import { Button } from "@chakra-ui/react";

type ButtonControlPanelProps = {
  children: React.ReactNode;
}

export function ButtonControlPanel({ children }: ButtonControlPanelProps) {

  return (
    <Button variant="ghost" width="100%" textAlign="left" justifyContent="flex-start">
      {children}
    </Button>
  )
}

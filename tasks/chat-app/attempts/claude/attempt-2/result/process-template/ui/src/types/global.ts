declare module "global" {
  global {
    interface Window {
      our: {
        node: string;
        process: string;
        address: string;  // Added address property which contains the full node address
      };
    }
  }
}

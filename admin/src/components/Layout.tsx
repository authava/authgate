import type { ReactNode } from 'react';
import { Link, useLocation } from 'react-router-dom';

interface LayoutProps {
  children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();

  return (
    <div className="min-h-screen flex flex-col">
      <header className="bg-gray-700 text-white shadow-md">
        <div className="container mx-auto px-4 py-4 flex justify-between items-center">
          <Link to="/" className="flex items-center space-x-2">
            <img src="https://authava.com/static/images/logo.svg" alt="Authava Logo" className="h-6 w-auto" />
            <span className="text-xl font-semibold text-white">AuthGate</span>
          </Link>
          <nav>
            <ul className="flex space-x-6">
              <li>
                <Link
                  to="/routes"
                  className={`hover:text-blue-200 ${location.pathname.startsWith('/routes') ? 'font-semibold' : ''}`}
                >
                  Routes
                </Link>
              </li>
            </ul>test
          </nav>
        </div>
      </header>

      <main className="flex-grow container mx-auto px-4 py-8">
        {children}
      </main>

      <footer className="bg-gray-100 border-t border-gray-200">
        <div className="container mx-auto px-4 py-4 text-center text-gray-600 text-sm">
          &copy; {new Date().getFullYear()} Authava, LLC. All rights reserved. | Privacy Policy | Terms of Service
        </div>
      </footer>
    </div>
  );
}
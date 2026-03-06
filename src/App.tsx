import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import LoginPage from "./pages/Login";
import DashboardPage from "./pages/Dashboard";
import FarmPage from "./pages/Farm";
import FriendsPage from "./pages/Friends";
import InventoryPage from "./pages/Inventory";
import TasksPage from "./pages/Tasks";
import SettingsPage from "./pages/Settings";
import LogsPage from "./pages/Logs";

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route element={<Layout />}>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/farm" element={<FarmPage />} />
          <Route path="/friends" element={<FriendsPage />} />
          <Route path="/inventory" element={<InventoryPage />} />
          <Route path="/tasks" element={<TasksPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/logs" element={<LogsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

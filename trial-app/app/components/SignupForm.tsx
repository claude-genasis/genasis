"use client";

import { useRouter } from "next/navigation";
import { useState, type FormEvent } from "react";

import { useLang } from "@/app/components/LangProvider";

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

const TECH_OPTIONS = [
  "React",
  "Next.js",
  "Vue",
  "Node",
  "Python",
  "Rust",
  "Go",
  "Mobile",
] as const;

const TEAM_SIZES = [
  { value: "solo" as const, labelKey: "signup.teamSize.solo" },
  { value: "small" as const, labelKey: "signup.teamSize.small" },
  { value: "medium" as const, labelKey: "signup.teamSize.medium" },
];

type TeamSize = (typeof TEAM_SIZES)[number]["value"];

type FormState = {
  name: string;
  email: string;
  phone: string;
  projectName: string;
  teamSize: TeamSize | "";
  techStack: string[];
  message: string;
};

const INITIAL: FormState = {
  name: "",
  email: "",
  phone: "",
  projectName: "",
  teamSize: "",
  techStack: [],
  message: "",
};

type Errors = Partial<Record<keyof FormState, string>>;
type Touched = Partial<Record<keyof FormState, boolean>>;

function techStackSlug(option: string): string {
  return option.toLowerCase().replace(/\./g, "").replace(/\s+/g, "-");
}

const inputClass =
  "w-full rounded-md border border-neutral-300 bg-white px-3 py-2 text-sm shadow-sm placeholder:text-neutral-400 focus:border-neutral-500 focus:outline-none focus:ring-2 focus:ring-neutral-200 dark:border-neutral-700 dark:bg-neutral-950 dark:text-neutral-100 dark:placeholder:text-neutral-600 dark:focus:ring-neutral-700";
const labelClass =
  "block text-sm font-medium text-neutral-800 dark:text-neutral-100";
const errorClass = "text-xs text-red-600 dark:text-red-400";

const requiredMarker = (
  <span className="ml-1 text-red-600 dark:text-red-400" aria-hidden="true">
    *
  </span>
);

export function SignupForm() {
  const router = useRouter();
  const { t } = useLang();
  const [form, setForm] = useState<FormState>(INITIAL);
  const [touched, setTouched] = useState<Touched>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const optionalMarker = (
    <span className="ml-1 text-xs font-normal text-neutral-500 dark:text-neutral-400">
      {t("signup.optional")}
    </span>
  );

  const validate = (f: FormState): Errors => {
    const errors: Errors = {};
    if (!f.name.trim()) errors.name = t("signup.required.name");
    if (!f.email.trim()) errors.email = t("signup.required.email");
    else if (!EMAIL_RE.test(f.email.trim()))
      errors.email = t("signup.required.email.format");
    if (!f.projectName.trim())
      errors.projectName = t("signup.required.projectName");
    if (!f.teamSize) errors.teamSize = t("signup.required.teamSize");
    return errors;
  };

  const errors = validate(form);
  const isValid = Object.keys(errors).length === 0;

  const update = <K extends keyof FormState>(key: K, value: FormState[K]) =>
    setForm((prev) => ({ ...prev, [key]: value }));

  const markTouched = (key: keyof FormState) =>
    setTouched((prev) => ({ ...prev, [key]: true }));

  const toggleTech = (opt: string) =>
    setForm((prev) => ({
      ...prev,
      techStack: prev.techStack.includes(opt)
        ? prev.techStack.filter((x) => x !== opt)
        : [...prev.techStack, opt],
    }));

  const showError = (key: keyof FormState) =>
    Boolean(touched[key]) && Boolean(errors[key]);

  const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!isValid) {
      setTouched({
        name: true,
        email: true,
        projectName: true,
        teamSize: true,
      });
      return;
    }
    setSubmitting(true);
    setSubmitError(null);
    try {
      const res = await fetch("/api/submit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: form.name.trim(),
          email: form.email.trim(),
          phone: form.phone.trim() || undefined,
          projectName: form.projectName.trim(),
          teamSize: form.teamSize,
          techStack: form.techStack,
          message: form.message.trim() || undefined,
        }),
      });
      const data = (await res.json().catch(() => ({}))) as {
        statusUrl?: string;
        error?: string;
      };
      if (data.statusUrl) {
        router.push(data.statusUrl);
        return;
      }
      if (!res.ok) {
        setSubmitError(
          data.error === "validation_failed"
            ? t("signup.submit.error.validation")
            : t("signup.submit.error.generic"),
        );
      }
    } catch {
      setSubmitError(t("signup.submit.error.network"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form
      data-testid="signup-form"
      onSubmit={handleSubmit}
      noValidate
      className="mx-auto max-w-2xl space-y-5"
    >
      <div className="space-y-1.5">
        <label htmlFor="signup-name" className={labelClass}>
          {t("signup.field.name")}
          {requiredMarker}
        </label>
        <input
          id="signup-name"
          data-testid="field-name"
          type="text"
          autoComplete="name"
          value={form.name}
          onChange={(e) => update("name", e.target.value)}
          onBlur={() => markTouched("name")}
          required
          aria-required="true"
          aria-invalid={showError("name") || undefined}
          aria-describedby={showError("name") ? "error-name" : undefined}
          className={inputClass}
        />
        {showError("name") ? (
          <p
            id="error-name"
            data-testid="error-name"
            role="alert"
            className={errorClass}
          >
            {errors.name}
          </p>
        ) : null}
      </div>

      <div className="space-y-1.5">
        <label htmlFor="signup-email" className={labelClass}>
          {t("signup.field.email")}
          {requiredMarker}
        </label>
        <input
          id="signup-email"
          data-testid="field-email"
          type="email"
          autoComplete="email"
          value={form.email}
          onChange={(e) => update("email", e.target.value)}
          onBlur={() => markTouched("email")}
          required
          aria-required="true"
          aria-invalid={showError("email") || undefined}
          aria-describedby={showError("email") ? "error-email" : undefined}
          placeholder={t("signup.placeholder.email")}
          className={inputClass}
        />
        {showError("email") ? (
          <p
            id="error-email"
            data-testid="error-email"
            role="alert"
            className={errorClass}
          >
            {errors.email}
          </p>
        ) : null}
      </div>

      <div className="space-y-1.5">
        <label htmlFor="signup-phone" className={labelClass}>
          {t("signup.field.phone")}
          {optionalMarker}
        </label>
        <input
          id="signup-phone"
          data-testid="field-phone"
          type="tel"
          autoComplete="tel"
          value={form.phone}
          onChange={(e) => update("phone", e.target.value)}
          placeholder={t("signup.placeholder.phone")}
          className={inputClass}
        />
      </div>

      <div className="space-y-1.5">
        <label htmlFor="signup-project-name" className={labelClass}>
          {t("signup.field.projectName")}
          {requiredMarker}
        </label>
        <input
          id="signup-project-name"
          data-testid="field-projectName"
          type="text"
          value={form.projectName}
          onChange={(e) => update("projectName", e.target.value)}
          onBlur={() => markTouched("projectName")}
          required
          aria-required="true"
          aria-invalid={showError("projectName") || undefined}
          aria-describedby={
            showError("projectName") ? "error-projectName" : undefined
          }
          placeholder={t("signup.placeholder.projectName")}
          className={inputClass}
        />
        {showError("projectName") ? (
          <p
            id="error-projectName"
            data-testid="error-projectName"
            role="alert"
            className={errorClass}
          >
            {errors.projectName}
          </p>
        ) : null}
      </div>

      <div className="space-y-1.5">
        <label htmlFor="signup-team-size" className={labelClass}>
          {t("signup.field.teamSize")}
          {requiredMarker}
        </label>
        <select
          id="signup-team-size"
          data-testid="field-teamSize"
          value={form.teamSize}
          onChange={(e) =>
            update("teamSize", e.target.value as TeamSize | "")
          }
          onBlur={() => markTouched("teamSize")}
          required
          aria-required="true"
          aria-invalid={showError("teamSize") || undefined}
          aria-describedby={
            showError("teamSize") ? "error-teamSize" : undefined
          }
          className={inputClass}
        >
          <option value="">{t("signup.teamSize.placeholder")}</option>
          {TEAM_SIZES.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {t(opt.labelKey)}
            </option>
          ))}
        </select>
        {showError("teamSize") ? (
          <p
            id="error-teamSize"
            data-testid="error-teamSize"
            role="alert"
            className={errorClass}
          >
            {errors.teamSize}
          </p>
        ) : null}
      </div>

      <fieldset data-testid="field-techStack" className="space-y-2">
        <legend className={labelClass}>
          {t("signup.field.techStack")}
          {optionalMarker}
        </legend>
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          {TECH_OPTIONS.map((opt) => {
            const checked = form.techStack.includes(opt);
            const slug = techStackSlug(opt);
            const id = `techstack-${slug}`;
            return (
              <label
                key={opt}
                htmlFor={id}
                className={`flex cursor-pointer select-none items-center gap-2 rounded-md border px-3 py-2 text-sm shadow-sm transition-colors ${
                  checked
                    ? "border-neutral-900 bg-neutral-900 text-white dark:border-white dark:bg-white dark:text-neutral-900"
                    : "border-neutral-200 bg-white hover:bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-950 dark:hover:bg-neutral-900"
                }`}
              >
                <input
                  id={id}
                  data-testid={id}
                  type="checkbox"
                  checked={checked}
                  onChange={() => toggleTech(opt)}
                  className="h-4 w-4 rounded border-neutral-300 accent-current"
                />
                <span>{opt}</span>
              </label>
            );
          })}
        </div>
      </fieldset>

      <div className="space-y-1.5">
        <label htmlFor="signup-message" className={labelClass}>
          {t("signup.field.message")}
          {optionalMarker}
        </label>
        <textarea
          id="signup-message"
          data-testid="field-message"
          rows={3}
          value={form.message}
          onChange={(e) => update("message", e.target.value)}
          placeholder={t("signup.placeholder.message")}
          className={inputClass}
        />
      </div>

      <div className="space-y-3 pt-1">
        <button
          type="submit"
          disabled={!isValid || submitting}
          data-testid="signup-submit"
          className="w-full rounded-md bg-neutral-900 px-4 py-2.5 text-sm font-medium text-white shadow-sm transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:bg-neutral-300 disabled:text-neutral-600 dark:bg-white dark:text-neutral-900 dark:hover:bg-neutral-200 dark:disabled:bg-neutral-700 dark:disabled:text-neutral-400"
        >
          {submitting ? t("signup.submit.sending") : t("signup.submit.idle")}
        </button>
        {submitError ? (
          <p
            data-testid="signup-submit-error"
            role="alert"
            className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300"
          >
            {submitError}
          </p>
        ) : null}
        <p
          data-testid="signup-info-banner"
          role="note"
          className="rounded-md border border-blue-200 bg-blue-50 px-3 py-2 text-xs text-blue-900 dark:border-blue-900 dark:bg-blue-950/50 dark:text-blue-200"
        >
          {t("signup.banner")}
        </p>
      </div>
    </form>
  );
}

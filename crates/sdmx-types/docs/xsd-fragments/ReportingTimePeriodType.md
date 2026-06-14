<details>
<summary>XSD contract: <code>ReportingTimePeriodType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="ReportingTimePeriodType">
		<xs:annotation>
			<xs:documentation>ReportingTimePeriodType defines standard reporting periods in SDMX, which are all in relation to the start day (day-month) of a reporting year which is specified in the specialized reporting year start day attribute. If the reporting year start day is not defined, a day of January 1 is assumed. The reporting year must be expressed as the year at the beginning of the period. Therefore, if the reporting year runs from April to March, any given reporting year is expressed as the year for April. The general format of a report period can be described as [year]-[period][time zone]?, where the type of period is designated with a single character followed by a number representing the period. Note that all periods allow for an optional time zone offset. See the details of each member type for the specifics of its format.</xs:documentation>
		</xs:annotation>
		<xs:union memberTypes="ReportingYearType ReportingSemesterType ReportingTrimesterType ReportingQuarterType ReportingMonthType ReportingWeekType ReportingDayType"/>
	</xs:simpleType>
```

</details>
